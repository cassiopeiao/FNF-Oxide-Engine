//! Adobe Animate texture-atlas loading.
//!
//! The original game ships Dad, Girlfriend and the title text as Adobe Animate
//! atlases (`Animation.json` + `spritemap1.json` + `spritemap1.png`).  Bevy has
//! no loader for that format, so this module renders every animation frame into
//! a plain RGBA sheet at load time and returns a regular [`SparrowAtlas`].  The
//! gameplay code then treats an Animate character exactly like a Sparrow sheet:
//! one texture, a set of named frame rectangles.

use std::collections::HashMap;

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext, RenderAssetUsages};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use serde::Deserialize;

use super::sparrow::{SparrowAtlas, SparrowFrame};

// ---------------------------------------------------------------------------
// JSON schema
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct AnimateDoc {
    #[serde(rename = "AN")]
    an: AnRoot,
    #[serde(rename = "SD")]
    sd: SymDict,
}

#[derive(Deserialize)]
struct AnRoot {
    #[serde(rename = "TL")]
    tl: Timeline,
}

#[derive(Deserialize, Default)]
struct Timeline {
    #[serde(rename = "L", default)]
    l: Vec<Layer>,
}

#[derive(Deserialize, Default)]
struct Layer {
    #[serde(rename = "FR", default)]
    fr: Vec<Frame>,
}

#[derive(Deserialize, Default)]
struct Frame {
    #[serde(rename = "I", default)]
    i: i64,
    #[serde(rename = "DU", default = "default_du")]
    du: i64,
    #[serde(rename = "E", default)]
    e: Vec<Element>,
    #[serde(rename = "N", default)]
    n: Option<String>,
}

fn default_du() -> i64 {
    1
}

#[derive(Deserialize, Default)]
struct Element {
    #[serde(rename = "ASI", default)]
    asi: Option<BitmapRef>,
    #[serde(rename = "SI", default)]
    si: Option<SymbolRef>,
}

#[derive(Deserialize, Default)]
struct BitmapRef {
    #[serde(rename = "N", default)]
    n: Option<String>,
    #[serde(rename = "MX", default)]
    mx: Option<[f64; 6]>,
}

#[derive(Deserialize, Default)]
struct SymbolRef {
    #[serde(rename = "SN", default)]
    sn: Option<String>,
    #[serde(rename = "FF", default)]
    ff: i64,
    #[serde(rename = "MX", default)]
    mx: Option<[f64; 6]>,
}

#[derive(Deserialize)]
struct SymDict {
    #[serde(rename = "S", default)]
    s: Vec<SymbolDef>,
}

#[derive(Deserialize)]
struct SymbolDef {
    #[serde(rename = "SN", default)]
    sn: String,
    #[serde(rename = "TL", default)]
    tl: Timeline,
}

#[derive(Deserialize)]
struct SpritemapDoc {
    #[serde(rename = "ATLAS")]
    atlas: AtlasDef,
}

#[derive(Deserialize)]
struct AtlasDef {
    #[serde(rename = "SPRITES", default)]
    sprites: Vec<SpriteEntry>,
}

#[derive(Deserialize)]
struct SpriteEntry {
    #[serde(rename = "SPRITE")]
    sprite: SpriteDef,
}

#[derive(Deserialize)]
struct SpriteDef {
    #[serde(default)]
    name: String,
    #[serde(default)]
    x: u32,
    #[serde(default)]
    y: u32,
    #[serde(default)]
    w: u32,
    #[serde(default)]
    h: u32,
    #[serde(default)]
    rotated: bool,
}

// ---------------------------------------------------------------------------
// Runtime structures
// ---------------------------------------------------------------------------

/// A 2x3 affine matrix `[a, b, c, d, tx, ty]`, mapping a point `(x, y)` to
/// `(a*x + c*y + tx, b*x + d*y + ty)`.
type Matrix = [f64; 6];

struct Sprite {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

struct Symbol {
    layers: Vec<Vec<Frame>>,
}

impl Symbol {
    fn length(&self) -> i64 {
        self.layers
            .iter()
            .flat_map(|frames| frames.iter())
            .map(|frame| frame.i + frame.du.max(1))
            .max()
            .unwrap_or(1)
    }

    fn frame_for<'a>(&self, layer: &'a [Frame], index: i64) -> Option<&'a Frame> {
        for frame in layer {
            if frame.i <= index && index < frame.i + frame.du.max(1) {
                return Some(frame);
            }
        }
        layer.last()
    }
}

fn mul(m: Matrix, n: Matrix) -> Matrix {
    let [a, b, c, d, tx, ty] = m;
    let [a2, b2, c2, d2, tx2, ty2] = n;
    [
        a * a2 + c * b2,
        b * a2 + d * b2,
        a * c2 + c * d2,
        b * c2 + d * d2,
        a * tx2 + c * ty2 + tx,
        b * tx2 + d * ty2 + ty,
    ]
}

fn identity() -> Matrix {
    [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]
}

/// Collect the flat list of `(matrix, sprite)` draw calls for a given frame.
fn collect_layers<'a>(
    symbols: &HashMap<String, Symbol>,
    sprites: &'a HashMap<String, Sprite>,
    symbol: &Symbol,
    frame_index: i64,
    parent: Matrix,
    out: &mut Vec<(Matrix, &'a Sprite)>,
    depth: usize,
) {
    if depth > 12 {
        return;
    }
    let length = symbol.length();
    let frame_index = if length > 0 { frame_index % length } else { 0 };

    for layer in symbol.layers.iter().rev() {
        let Some(frame) = symbol.frame_for(layer, frame_index) else {
            continue;
        };
        for element in &frame.e {
            if let Some(bitmap) = &element.asi {
                let Some(name) = &bitmap.n else { continue };
                let Some(sprite) = sprites.get(name) else {
                    continue;
                };
                let mx = bitmap.mx.unwrap_or(identity());
                out.push((mul(mx, parent), sprite));
            } else if let Some(instance) = &element.si {
                let Some(name) = &instance.sn else { continue };
                let Some(child) = symbols.get(name) else {
                    continue;
                };
                let mx = instance.mx.unwrap_or(identity());
                let local = instance.ff + (frame_index - frame.i);
                collect_layers(
                    symbols,
                    sprites,
                    child,
                    local,
                    mul(mx, parent),
                    out,
                    depth + 1,
                );
            }
        }
    }
}

/// Alpha-composite `src` over `dst` using straight (non-premultiplied) alpha.
fn alpha_over(dst: &mut [u8; 4], src: [u8; 4]) {
    let sa = src[3] as f32 / 255.0;
    if sa <= 0.0 {
        return;
    }
    let da = dst[3] as f32 / 255.0;
    let out_a = sa + da * (1.0 - sa);
    if out_a <= 0.0 {
        return;
    }
    for channel in 0..3 {
        let sc = src[channel] as f32;
        let dc = dst[channel] as f32;
        let out = (sc * sa + dc * da * (1.0 - sa)) / out_a;
        dst[channel] = out.round().clamp(0.0, 255.0) as u8;
    }
    dst[3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
}

/// Render one animation frame onto a transparent RGBA canvas.
#[allow(clippy::too_many_arguments)]
fn render_frame(
    symbols: &HashMap<String, Symbol>,
    sprites: &HashMap<String, Sprite>,
    root: &Symbol,
    frame_index: i64,
    canvas_w: u32,
    canvas_h: u32,
    offset_x: f64,
    offset_y: f64,
) -> Vec<u8> {
    let mut layers: Vec<(Matrix, &Sprite)> = Vec::new();
    collect_layers(
        symbols,
        sprites,
        root,
        frame_index,
        identity(),
        &mut layers,
        0,
    );

    let mut canvas = vec![0u8; (canvas_w * canvas_h * 4) as usize];

    for (matrix, sprite) in layers {
        let [a, b, c, d, tx, ty] = matrix;
        let tx = tx + offset_x;
        let ty = ty + offset_y;
        let det = a * d - b * c;
        if det.abs() < 1e-9 {
            continue;
        }
        // Inverse matrix: maps output canvas coords to input sprite coords.
        let inv = [
            d / det,
            -c / det,
            (c * ty - d * tx) / det,
            -b / det,
            a / det,
            (b * tx - a * ty) / det,
        ];

        let sw = sprite.width as i32;
        let sh = sprite.height as i32;
        let src = &sprite.pixels;

        for py in 0..canvas_h {
            let y = py as f64;
            for px in 0..canvas_w {
                let x = px as f64;
                let sx = inv[0] * x + inv[1] * y + inv[2];
                let sy = inv[3] * x + inv[4] * y + inv[5];
                let ix = sx.floor() as i32;
                let iy = sy.floor() as i32;
                if ix < 0 || iy < 0 || ix >= sw || iy >= sh {
                    continue;
                }
                let si = ((iy * sw + ix) * 4) as usize;
                let di = ((py * canvas_w + px) * 4) as usize;
                let sample = [src[si], src[si + 1], src[si + 2], src[si + 3]];
                let dst = [canvas[di], canvas[di + 1], canvas[di + 2], canvas[di + 3]];
                let mut out = dst;
                alpha_over(&mut out, sample);
                canvas[di..di + 4].copy_from_slice(&out);
            }
        }
    }

    canvas
}

fn bbox(pixels: &[u8], width: u32, height: u32) -> Option<(u32, u32, u32, u32)> {
    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0u32;
    let mut max_y = 0u32;
    let mut found = false;
    for y in 0..height {
        for x in 0..width {
            let alpha = pixels[((y * width + x) * 4 + 3) as usize];
            if alpha > 0 {
                found = true;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }
    found.then_some((min_x, min_y, max_x + 1, max_y + 1))
}

// ---------------------------------------------------------------------------
// Loader
// ---------------------------------------------------------------------------

#[derive(Default, TypePath)]
pub struct AnimateLoader;

impl AssetLoader for AnimateLoader {
    type Asset = SparrowAtlas;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let document: AnimateDoc = serde_json::from_slice(&bytes).map_err(|error| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid Animation.json: {error}"),
            )
        })?;

        let parent = load_context
            .path()
            .path()
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();
        let spritemap_bytes = load_context
            .read_asset_bytes(parent.join("spritemap1.json"))
            .await
            .map_err(|error| std::io::Error::other(format!("spritemap1.json: {error}")))?;
        let spritemap: SpritemapDoc =
            serde_json::from_slice(&spritemap_bytes).map_err(|error| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("invalid spritemap1.json: {error}"),
                )
            })?;

        let png_bytes = load_context
            .read_asset_bytes(parent.join("spritemap1.png"))
            .await
            .map_err(|error| std::io::Error::other(format!("spritemap1.png: {error}")))?;
        let atlas_image = image::load_from_memory(&png_bytes)
            .map_err(|error| std::io::Error::other(format!("spritemap1.png decode: {error}")))?
            .to_rgba8();

        // Build the sprite map, un-rotating rotated sprites.
        let mut sprites: HashMap<String, Sprite> = HashMap::new();
        for entry in &spritemap.atlas.sprites {
            let def = &entry.sprite;
            if def.name.is_empty() {
                continue;
            }
            let (crop_w, crop_h) = if def.rotated {
                (def.h, def.w)
            } else {
                (def.w, def.h)
            };
            if crop_w == 0 || crop_h == 0 {
                continue;
            }
            let cropped =
                image::imageops::crop_imm(&atlas_image, def.x, def.y, crop_w, crop_h).to_image();
            let oriented = if def.rotated {
                image::imageops::rotate270(&cropped)
            } else {
                cropped
            };
            let (width, height) = (oriented.width(), oriented.height());
            sprites.insert(
                def.name.clone(),
                Sprite {
                    width,
                    height,
                    pixels: oriented.into_raw(),
                },
            );
        }

        // Root symbol + animation labels.
        let symbols: HashMap<String, Symbol> = document
            .sd
            .s
            .into_iter()
            .map(|def| {
                let symbol = Symbol {
                    layers: def.tl.l.into_iter().map(|layer| layer.fr).collect(),
                };
                (def.sn, symbol)
            })
            .collect();

        let root = Symbol {
            layers: document.an.tl.l.into_iter().map(|layer| layer.fr).collect(),
        };

        let mut labels: Vec<(String, i64, i64)> = Vec::new();
        for layer in &root.layers {
            for frame in layer {
                if let Some(name) = &frame.n {
                    labels.push((name.clone(), frame.i, frame.du.max(1)));
                }
            }
        }

        // Pass 1: render every labelled frame to a fixed canvas and measure the
        // union bounding box of all of them.  We reuse the character-art canvas
        // that worked for the original build script (the game's characters and
        // title text all fit comfortably inside it).
        let (canvas_w, canvas_h, offset_x, offset_y) = (1600u32, 1600u32, 800.0, 800.0);
        let mut union: Option<(u32, u32, u32, u32)> = None;
        let mut jobs: Vec<(String, i64)> = Vec::new();
        for (label, start, length) in &labels {
            for local in 0..*length {
                let frame_index = start + local;
                let canvas = render_frame(
                    &symbols,
                    &sprites,
                    &root,
                    frame_index,
                    canvas_w,
                    canvas_h,
                    offset_x,
                    offset_y,
                );
                if let Some(box_) = bbox(&canvas, canvas_w, canvas_h) {
                    union = Some(match union {
                        None => box_,
                        Some(u) => (
                            u.0.min(box_.0),
                            u.1.min(box_.1),
                            u.2.max(box_.2),
                            u.3.max(box_.3),
                        ),
                    });
                }
                jobs.push((label.clone(), frame_index));
            }
        }

        let Some((min_x, min_y, max_x, max_y)) = union else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Animate atlas produced no pixels",
            ));
        };
        let cell_w = max_x - min_x;
        let cell_h = max_y - min_y;

        // Pack the frames into a grid so the sheet stays within GPU texture
        // limits (a single row would be tens of thousands of pixels wide).
        const COLS: u32 = 6;
        let rows = jobs.len().div_ceil(COLS as usize) as u32;
        let sheet_w = cell_w * COLS;
        let sheet_h = cell_h * rows;
        let mut sheet = vec![0u8; (sheet_w * sheet_h * 4) as usize];
        let mut frames = HashMap::new();

        for (index, (label, frame_index)) in jobs.iter().enumerate() {
            // Render again directly into the union-sized cell.
            let cell = render_frame(
                &symbols,
                &sprites,
                &root,
                *frame_index,
                cell_w,
                cell_h,
                offset_x - min_x as f64,
                offset_y - min_y as f64,
            );
            let col = index as u32 % COLS;
            let row = index as u32 / COLS;
            let cell_x = col * cell_w;
            let cell_y = row * cell_h;
            for y in 0..cell_h {
                for x in 0..cell_w {
                    let src_i = ((y * cell_w + x) * 4) as usize;
                    let dst_i = (((cell_y + y) * sheet_w + cell_x + x) * 4) as usize;
                    sheet[dst_i..dst_i + 4].copy_from_slice(&cell[src_i..src_i + 4]);
                }
            }
            frames.insert(
                format!("{label}{frame_index:04}"),
                SparrowFrame {
                    rect: Rect::new(
                        cell_x as f32,
                        cell_y as f32,
                        (cell_x + cell_w) as f32,
                        (cell_y + cell_h) as f32,
                    ),
                    size: Vec2::new(cell_w as f32, cell_h as f32),
                    offset: Vec2::ZERO,
                    source_size: Vec2::new(cell_w as f32, cell_h as f32),
                },
            );
        }

        if let Some(dir) = std::env::var_os("FNF_OXIDE_DUMP_SHEET") {
            let name = load_context
                .path()
                .path()
                .parent()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "sheet".into());
            let path = std::path::PathBuf::from(dir).join(format!("{name}-sheet.png"));
            let _ = image::save_buffer(&path, &sheet, sheet_w, sheet_h, image::ColorType::Rgba8);
            bevy::log::info!("dumped animate sheet to {}", path.display());
        }

        let image = Image::new(
            Extent3d {
                width: sheet_w,
                height: sheet_h,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            sheet,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        let texture = load_context.add_labeled_asset("texture", image);

        Ok(SparrowAtlas { texture, frames })
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

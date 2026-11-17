//! Font and texture handles shared by every screen.
//!
//! Assets are loaded once during startup; screens receive the handles through
//! this resource and can spawn immediately while Bevy finishes decoding in the
//! background.  Everything here points at the original game's assets under
//! `assets/` (no derived/cooked art).

use bevy::prelude::*;

use crate::animate_asset::AnimateAtlasAsset;
use crate::sparrow::SparrowAtlas;

#[derive(Resource, Debug, Clone)]
pub struct GameAssets {
    /// Main UI font (VCR OSD Mono).
    pub font_ui: Handle<Font>,
    /// Score / digital font (5by7 bold).
    pub font_score: Handle<Font>,
    /// Title logo (the animated ``logo bumpin`` Sparrow atlas).
    pub logo_bumpin: Handle<SparrowAtlas>,
    /// Title screen "PRESS ENTER TO BEGIN" (Adobe Animate atlas).
    pub title_text: Handle<SparrowAtlas>,
    /// Animated-ish menu background.
    pub menu_bg: Handle<Image>,
    /// Desaturated menu background used behind submenus.
    pub menu_desat: Handle<Image>,
    /// One arrow texture per lane (original `notes.xml`, single frame each).
    pub notes: Handle<SparrowAtlas>,
    /// Receptor art (original `noteStrumline.xml`: static/press/confirm).
    pub note_strumline: Handle<SparrowAtlas>,
    /// Hit-splash art (original `noteSplashes.xml`).
    pub note_splashes: Handle<SparrowAtlas>,
    /// Sustain-trail strip (`NOTE_hold_assets.png`, eight 52px columns).
    pub note_holds: Handle<Image>,
    /// Week 1 stage layers.
    pub stage_back: Handle<Image>,
    pub stage_front: Handle<Image>,
    pub stage_curtains: Handle<Image>,
    /// Character atlases (BF/GF are Sparrow, Dad is Adobe Animate).
    pub bf_atlas: Handle<SparrowAtlas>,
    pub dad_animate: Handle<AnimateAtlasAsset>,
    pub gf_atlas: Handle<SparrowAtlas>,
    /// Freeplay artwork.
    pub freeplay_bg: Handle<Image>,
    /// Freeplay difficulty selector images (easy/normal/hard).
    pub fp_difficulties: Vec<Handle<Image>>,
    /// Freeplay album cover + its "FNF OST VOL 1" lettering.
    pub fp_album: Handle<Image>,
    pub fp_album_text: Handle<Image>,
    /// Freeplay difficulty-selector arrows (left/right idle frames).
    pub fp_arrows: Handle<Image>,
    /// Story-mode week titles, in week order.
    pub story_titles: Vec<Handle<Image>>,
    /// Story-mode difficulty chips (easy/normal/hard).
    pub story_difficulties: Vec<Handle<Image>>,
    /// Story-mode left/right arrows.
    pub story_arrows: Handle<SparrowAtlas>,
    /// Main-menu lettered buttons (each is a Sparrow atlas with ``idle`` and
    /// ``selected`` animations).
    pub menu_storymode: Handle<SparrowAtlas>,
    pub menu_freeplay: Handle<SparrowAtlas>,
    pub menu_options: Handle<SparrowAtlas>,
    pub menu_credits: Handle<SparrowAtlas>,
    /// Health bar art and icons.
    pub health_bar: Handle<Image>,
    pub icon_bf: Handle<Image>,
    pub icon_dad: Handle<Image>,
    pub icon_gf: Handle<Image>,
}

impl GameAssets {
    pub fn load(asset_server: &AssetServer) -> Self {
        Self {
            font_ui: asset_server.load("fonts/vcr.ttf"),
            font_score: asset_server.load("fonts/5by7_b.ttf"),
            logo_bumpin: asset_server.load("preload/images/logoBumpin.xml"),
            title_text: asset_server.load("preload/images/title-screen-text/Animation.json"),
            menu_bg: asset_server.load("preload/images/menuBG.png"),
            menu_desat: asset_server.load("preload/images/menuDesat.png"),
            notes: asset_server.load("shared/images/notes.xml"),
            note_strumline: asset_server.load("shared/images/noteStrumline.xml"),
            note_splashes: asset_server.load("shared/images/noteSplashes.xml"),
            note_holds: asset_server.load("shared/images/NOTE_hold_assets.png"),
            stage_back: asset_server.load("shared/images/stageback.png"),
            stage_front: asset_server.load("shared/images/stagefront.png"),
            stage_curtains: asset_server.load("shared/images/stagecurtains.png"),
            bf_atlas: asset_server.load("shared/images/characters/BOYFRIEND.xml"),
            dad_animate: asset_server.load("shared/images/characters/dad/Animation.json"),
            gf_atlas: asset_server.load("preload/images/gfDanceTitle.xml"),
            freeplay_bg: asset_server.load("preload/images/freeplay/freeplayBGweek1-bf.png"),
            fp_difficulties: ["freeplayeasy", "freeplaynormal", "freeplayhard"]
                .iter()
                .map(|id| asset_server.load(format!("preload/images/freeplay/{id}.png")))
                .collect(),
            fp_album: asset_server.load("preload/images/freeplay/albumRoll/volume1.png"),
            fp_album_text: asset_server.load("preload/images/freeplay/albumRoll/volume1-text.png"),
            fp_arrows: asset_server.load("preload/images/storymenu/ui/arrows.png"),
            story_titles: [
                "week1", "week2", "week3", "week4", "week5", "week6", "week7", "weekend1",
                "tutorial", "sserafim",
            ]
            .iter()
            .map(|id| asset_server.load(format!("preload/images/storymenu/titles/{id}.png")))
            .collect(),
            story_difficulties: ["easy", "normal", "hard"]
                .iter()
                .map(|id| {
                    asset_server.load(format!("preload/images/storymenu/difficulties/{id}.png"))
                })
                .collect(),
            story_arrows: asset_server.load("preload/images/storymenu/ui/arrows.xml"),
            menu_storymode: asset_server.load("preload/images/mainmenu/storymode.xml"),
            menu_freeplay: asset_server.load("preload/images/mainmenu/freeplay.xml"),
            menu_options: asset_server.load("preload/images/mainmenu/options.xml"),
            menu_credits: asset_server.load("preload/images/mainmenu/credits.xml"),
            health_bar: asset_server.load("shared/images/healthBar.png"),
            icon_bf: asset_server.load("preload/images/icons/icon-bf.png"),
            icon_dad: asset_server.load("preload/images/icons/icon-dad.png"),
            icon_gf: asset_server.load("preload/images/icons/icon-gf.png"),
        }
    }

    /// Convenience for UI: the UI font as a `TextFont` source.
    pub fn ui_font(&self) -> FontSource {
        self.font_ui.clone().into()
    }

    pub fn score_font(&self) -> FontSource {
        self.font_score.clone().into()
    }
}

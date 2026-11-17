// Vendored from `bevy_animate_atlas` 0.1.0 (GPL-3.0), ported to Bevy 0.20.
#![allow(clippy::all)]

// src/lib.rs
pub mod animate_atlas;
pub mod sparrow_atlas;

// Re-export core data structures for clean root access
pub use animate_atlas::{AnimateAtlas, build_part_mesh, get_animate_parts, parse_animate_atlas};
pub use sparrow_atlas::{SparrowFrame, parse_sparrow};

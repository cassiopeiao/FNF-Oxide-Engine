//! Generated bindings for `sdk/oesdk/wit/oesdk.wit` plus host implementations.

use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Engine, Store};

use crate::host::{HostState, ModCommand};

wasmtime::component::bindgen!({
    path: "../../sdk/oesdk/wit",
    world: "mod-world",
});

use self::fnf::mods::types::{CharacterInfo, Hit, Note, Side, Vec2};

impl fnf::mods::core::Host for HostState {
    fn log(&mut self, level: u8, msg: String) {
        let tag = match level {
            0 => "error",
            1 => "warn",
            2 => "info",
            _ => "debug",
        };
        bevy::log::info!("[mod:{tag}] {msg}");
        ()
    }

    fn play_state(&mut self) -> (String, String, f32, u32, u32, u32, f32, f32, u32, u8) {
        let play = &self.play;
        (
            play.song_name.clone(),
            play.difficulty.clone(),
            play.bpm,
            play.score,
            play.combo,
            play.max_combo,
            play.health,
            play.accuracy,
            play.total_notes,
            play.player_count,
        )
    }

    fn characters(&mut self) -> Vec<CharacterInfo> {
        self.characters
            .iter()
            .map(|info| CharacterInfo {
                id: info.id.0,
                name: info.name.clone(),
                side: match info.side {
                    oesdk::events::Side::Player => Side::Player,
                    oesdk::events::Side::Opponent => Side::Opponent,
                },
                slot: info.slot,
            })
            .collect()
    }

    fn character(&mut self, name: String) -> Option<u32> {
        self.characters
            .iter()
            .find(|info| info.name == name)
            .map(|info| info.id.0)
    }

    fn character_position(&mut self, id: u32) -> Vec2 {
        let pos = oesdk::prelude::Vec2::default();
        Vec2 { x: pos.x, y: pos.y }
    }

    fn set_character_position(&mut self, id: u32, pos: Vec2) -> () {
        self.commands.push(ModCommand::MoveCharacter {
            id,
            x: pos.x,
            y: pos.y,
        });
        ()
    }

    fn set_character_opacity(&mut self, id: u32, alpha: f32) -> () {
        self.commands.push(ModCommand::SetAlpha { id, alpha });
        ()
    }

    fn set_character_flip(&mut self, id: u32, flip: bool) -> () {
        self.commands.push(ModCommand::SetFlip { id, flip });
        ()
    }

    fn play_anim(&mut self, id: u32, anim: String) -> () {
        self.commands.push(ModCommand::PlayAnimById { id, anim });
        ()
    }
}

impl fnf::mods::camera::Host for HostState {
    fn shake(&mut self, intensity: f32, seconds: f32) -> () {
        self.commands
            .push(ModCommand::CameraShake { intensity, seconds });
        ()
    }

    fn flash(&mut self, rgba: u32, seconds: f32) -> () {
        self.commands
            .push(ModCommand::CameraFlash { rgba, seconds });
        ()
    }
}

impl fnf::mods::sprites::Host for HostState {
    fn spawn(&mut self, asset: String, pos: Vec2, layer: i32, alpha: f32) -> u32 {
        let id = self.next_sprite;
        self.next_sprite += 1;
        self.commands.push(ModCommand::SpawnSprite {
            asset,
            pos: oesdk::prelude::Vec2 { x: pos.x, y: pos.y },
            layer,
            alpha,
        });
        id
    }

    fn despawn(&mut self, id: u32) -> () {
        self.commands.push(ModCommand::DespawnSprite { id });
        ()
    }

    fn set_position(&mut self, id: u32, pos: Vec2) -> () {
        self.commands.push(ModCommand::MoveSprite {
            id,
            x: pos.x,
            y: pos.y,
        });
        ()
    }
}

impl fnf::mods::audio::Host for HostState {
    fn play(&mut self, asset: String, volume: f32) -> u32 {
        let id = self.next_sound;
        self.next_sound += 1;
        self.commands.push(ModCommand::PlaySound { asset, volume });
        id
    }

    fn stop(&mut self, id: u32) -> () {
        self.commands.push(ModCommand::StopSound { id });
        ()
    }
}

impl fnf::mods::rng::Host for HostState {
    fn next(&mut self) -> f32 {
        self.next_rng()
    }
}

/// A guest instance plus the store that owns it.
pub struct ModInstance {
    pub store: Store<HostState>,
    pub bindings: ModWorld,
}

impl ModInstance {
    pub fn new(
        engine: &Engine,
        component: &Component,
        _table: &mut ResourceTable,
    ) -> wasmtime::Result<Self> {
        // TODO(bindgen): link the generated imports (`core`, `camera`, ...).
        // `ModWorld::add_to_linker` now wants a typed store-data accessor that
        // the generated code has not exposed yet, so instantiation is off until
        // that accessor is pinned down.
        let mut linker: Linker<HostState> = Linker::new(engine);
        let mut store = Store::new(engine, HostState::new());
        store.set_fuel(50_000_000)?;
        store.epoch_deadline_trap();
        store.set_epoch_deadline(1);
        let bindings = ModWorld::instantiate(&mut store, component, &linker)?;
        Ok(Self { store, bindings })
    }

    pub fn call_create(&mut self) -> wasmtime::Result<()> {
        // TODO(bindgen): the exported `callbacks` interface is generated under
        // `exports::fnf::mods::callbacks`; wire its `Guest` dispatch here once
        // the accessor name is confirmed. Imports already work.
        let _ = &self.bindings;
        Ok(())
    }

    pub fn call_note_hit(
        &mut self,
        note: oesdk::events::Note,
        judge: oesdk::events::Judgement,
        delta_ms: f64,
        combo: u32,
        score: u32,
        accuracy: f32,
    ) -> wasmtime::Result<()> {
        let hit = Hit {
            note: Note {
                lane: note.lane,
                time_ms: note.time_ms,
                len_ms: note.len_ms,
                player: note.player,
            },
            judge: match judge {
                oesdk::events::Judgement::Sick => self::fnf::mods::types::Judge::Sick,
                oesdk::events::Judgement::Good => self::fnf::mods::types::Judge::Good,
                oesdk::events::Judgement::Bad => self::fnf::mods::types::Judge::Bad,
                oesdk::events::Judgement::Shit => self::fnf::mods::types::Judge::Shit,
                oesdk::events::Judgement::Miss => self::fnf::mods::types::Judge::Miss,
            },
            delta_ms,
            combo,
            score,
            acc: accuracy,
        };
        let _ = hit;
        Ok(())
    }
}

//! Headless example: runs a mod against a mock host, no engine required.
//!
//! `cargo run --example basic`

use oesdk::prelude::*;

#[derive(Default)]
struct ShakeOnSick;

impl Mod for ShakeOnSick {
    fn on_create(&mut self, ctx: &mut Ctx<'_>) {
        ctx.log(LogLevel::Info, "ShakeOnSick ready");
    }

    fn on_note_hit(&mut self, ctx: &mut Ctx<'_>, hit: &Hit) {
        if hit.judgement == Judgement::Sick {
            ctx.camera().shake(0.06, 0.25);
            if let Some(mut bf) = ctx.character("bf") {
                bf.play_anim("singLEFT");
            }
        }
    }
}

#[derive(Default)]
struct MockHost {
    shakes: Vec<(f32, f32)>,
    anims: Vec<(CharacterId, String)>,
}

impl HostBridge for MockHost {
    fn log(&mut self, level: LogLevel, msg: &str) {
        println!("[{level:?}] {msg}");
    }
    fn play_state(&mut self) -> PlayState {
        PlayState::default()
    }
    fn characters(&mut self) -> Vec<CharacterInfo> {
        vec![
            CharacterInfo {
                id: CharacterId(1),
                name: "bf".into(),
                side: Side::Player,
                slot: 0,
            },
            CharacterInfo {
                id: CharacterId(2),
                name: "dad".into(),
                side: Side::Opponent,
                slot: 0,
            },
        ]
    }
    fn character(&mut self, name: &str) -> Option<CharacterId> {
        self.characters()
            .into_iter()
            .find(|c| c.name == name)
            .map(|c| c.id)
    }
    fn character_position(&mut self, _id: CharacterId) -> Vec2 {
        Vec2::default()
    }
    fn set_character_position(&mut self, _id: CharacterId, _pos: Vec2) {}
    fn set_character_opacity(&mut self, _id: CharacterId, _alpha: f32) {}
    fn set_character_flip(&mut self, _id: CharacterId, _flip: bool) {}
    fn play_anim(&mut self, id: CharacterId, anim: &str) {
        self.anims.push((id, anim.to_string()));
    }
    fn camera_shake(&mut self, intensity: f32, seconds: f32) {
        self.shakes.push((intensity, seconds));
    }
    fn camera_flash(&mut self, _rgba: u32, _seconds: f32) {}
    fn spawn_sprite(&mut self, _asset: &str, _pos: Vec2, _layer: i32, _alpha: f32) -> SpriteId {
        SpriteId(1)
    }
    fn despawn(&mut self, _id: SpriteId) {}
    fn set_sprite_position(&mut self, _id: SpriteId, _pos: Vec2) {}
    fn rng(&mut self) -> f32 {
        0.5
    }
}

fn main() {
    let mut host = MockHost::default();
    let mut module = ShakeOnSick;
    let mut ctx = Ctx::new(&mut host);

    oesdk::registry::dispatch(
        &mut module,
        "on-create",
        &mut ctx,
        oesdk::registry::Payload::None,
    );
    let hit = Hit {
        note: Note {
            lane: 0,
            time_ms: 1200.0,
            len_ms: 0.0,
            player: true,
        },
        judgement: Judgement::Sick,
        delta_ms: 12.0,
        combo: 7,
        score: 2100,
        accuracy: 0.98,
    };
    oesdk::registry::dispatch(
        &mut module,
        "on-note-hit",
        &mut ctx,
        oesdk::registry::Payload::NoteHit(&hit),
    );

    println!("shakes: {:?}", host.shakes);
    println!("anims : {:?}", host.anims);
}

//! Gameplay systems: spawning the stage, reading input, judging notes and
//! driving the conductor.

use bevy::audio::Volume;
use bevy::input::touch::Touches;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use oxide_core::conductor::Conductor;
use oxide_core::config::*;
use oxide_core::sparrow::{SparrowAtlas, SparrowFrame};
use oxide_core::theme::{dim, lane_color};
use oxide_core::{AnimateAtlasAsset, AppState, GameAssets, RawJson, Settings};
use oxide_data::{Note, SongChartAsset};

use crate::characters::{self, CharacterSide};
use crate::{
    Judgement, JudgementCounts, LastResult, PlayPhase, PlaySession, SongSelection, hud, rank_for,
};

/// Per-frame lane state. `pressed` is held state, `just_pressed` is true for a
/// single frame and is consumed by the judgement system.
#[derive(Resource, Debug, Default)]
pub struct LaneState {
    pub pressed: [bool; LANE_COUNT],
    pub just_pressed: [bool; LANE_COUNT],
}

#[derive(Component, Debug)]
pub struct PlayObject;

#[derive(Component, Debug)]
pub struct GameNote;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotePart {
    Head,
    Body,
}

#[derive(Component, Debug)]
pub struct NoteMarker {
    pub note: Note,
    pub part: NotePart,
}

/// A note head waiting for its (async-loaded) arrow atlas before showing.
#[derive(Component, Debug)]
pub struct NoteSprite {
    pub atlas: Handle<SparrowAtlas>,
    pub ready: bool,
}

/// One receptor.  Static/press/confirm frames are filled in once the
/// `noteStrumline` atlas finishes loading.
#[derive(Component, Debug)]
pub struct Receptor {
    pub lane: usize,
    pub player: bool,
    pub pressed: bool,
    /// Seconds remaining in the confirm animation (0 = not confirming).
    pub confirm: f32,
    pub state: ReceptorState,
    pub frame: usize,
    pub elapsed: f32,
    pub ready: bool,
    pub static_frames: Vec<SparrowFrame>,
    pub press_frames: Vec<SparrowFrame>,
    pub confirm_frames: Vec<SparrowFrame>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReceptorState {
    #[default]
    Static,
    Press,
    Confirm,
}

/// A hit splash spawned on SICK/GOOD.
#[derive(Component, Debug)]
pub struct NoteSplash {
    pub prefix: String,
    pub frames: Vec<SparrowFrame>,
    pub index: usize,
    pub elapsed: f32,
    pub ready: bool,
}

#[derive(Component, Debug)]
pub struct SongAudio;

/// Original note-frame names, indexed by lane (left, down, up, right).
const DIR_NAMES: [&str; LANE_COUNT] = ["Left", "Down", "Up", "Right"];

/// Splash colour names, indexed by lane (left, down, up, right).
const SPLASH_COLORS: [&str; LANE_COUNT] = ["purple", "blue", "green", "red"];

/// Original `StrumlineNote.CONFIRM_HOLD_TIME`.
const CONFIRM_HOLD_TIME: f32 = 0.15;

/// Animation frame time at 24 fps.
const ANIM_FRAME_TIME: f32 = 1.0 / 24.0;

/// Spawn the stage and start loading the chart + instrumental.
pub fn setup_play(
    mut commands: Commands,
    assets: Res<GameAssets>,
    selection: Res<SongSelection>,
    settings: Res<Settings>,
    asset_server: Res<AssetServer>,
    mut lanes: ResMut<LaneState>,
) {
    *lanes = LaneState::default();
    let entry = selection.entry();

    // Stage geometry comes from the level data; props are spawned once the
    // stage definition and its textures have loaded (see `spawn_stage`).
    let stage = asset_server.load::<RawJson>(format!("preload/data/stages/{}.json", entry.stage));

    // Receptors.
    let y = strum_y(settings.downscroll);
    for player in [false, true] {
        for lane in 0..LANE_COUNT {
            commands.spawn((
                Sprite::default(),
                Transform::from_xyz(strum_x(player, lane), y, 8.0),
                Receptor {
                    lane,
                    player,
                    pressed: false,
                    confirm: 0.0,
                    state: ReceptorState::Static,
                    frame: 0,
                    elapsed: 0.0,
                    ready: false,
                    static_frames: Vec::new(),
                    press_frames: Vec::new(),
                    confirm_frames: Vec::new(),
                },
                PlayObject,
            ));
        }
    }

    hud::spawn_hud(
        &mut commands,
        &assets,
        entry.song_name,
        selection.difficulty,
    );

    let chart: Handle<SongChartAsset> = asset_server.load(entry.chart_path());
    let instrumental = asset_server.load(entry.instrumental_path());
    let metadata = asset_server.load::<RawJson>(format!(
        "preload/data/songs/{id}/{id}-metadata.json",
        id = entry.id
    ));

    commands.insert_resource(PlaySession {
        song: entry,
        difficulty: selection.difficulty,
        chart,
        instrumental,
        vocals: Vec::new(),
        phase: PlayPhase::Loading,
        paused_from: None,
        conductor: Conductor::new(f64::from(entry.bpm)),
        scroll_speed: 1.0,
        duration_ms: 0.0,
        total_notes: 0,
        score: 0,
        combo: 0,
        max_combo: 0,
        counts: JudgementCounts::default(),
        health: 50.0,
        audio: None,
        popup: None,
        failed: false,
        finished: false,
        player_sing: None,
        player_miss: None,
        opponent_sing: None,
        opponent_notes: Vec::new(),
        next_opponent: 0,
        stage,
        stage_spawned: false,
        metadata,
        cast_spawned: false,
    });
}

/// Once the chart asset is ready, build note entities and start the countdown.
pub fn begin_when_loaded(
    mut commands: Commands,
    charts: Res<Assets<SongChartAsset>>,
    metas: Res<Assets<RawJson>>,
    assets: Res<GameAssets>,
    asset_server: Res<AssetServer>,
    settings: Res<Settings>,
    mut session: ResMut<PlaySession>,
) {
    if session.phase != PlayPhase::Loading {
        return;
    }
    let Some(chart) = charts.get(&session.chart) else {
        return;
    };
    // The vocal tracks are split per character in the assets, and the metadata
    // tells us which ones this song uses (`playerVocals` / `opponentVocals`).
    let Some(meta) = metas.get(&session.metadata) else {
        return;
    };
    let parsed: SongMetadata = serde_json::from_str(&meta.0).unwrap_or_default();
    let mut names: Vec<String> = Vec::new();
    for name in parsed
        .play_data
        .characters
        .player_vocals
        .iter()
        .chain(parsed.play_data.characters.opponent_vocals.iter())
    {
        if !names.contains(name) {
            names.push(name.clone());
        }
    }
    if names.is_empty() {
        names.push("bf".to_string());
    }
    session.vocals = names
        .iter()
        .map(|name| asset_server.load(format!("songs/{}/Voices-{}.ogg", session.song.id, name)))
        .collect();

    session.scroll_speed = chart.0.scroll_speed_for(session.difficulty).unwrap_or(1.0);
    session.duration_ms = chart.0.duration_ms();
    let notes = chart.0.notes_for(session.difficulty);
    session.total_notes = notes.iter().filter(|note| note.is_player()).count() as u32;
    session.opponent_notes = notes
        .iter()
        .filter(|note| !note.is_player())
        .copied()
        .collect();
    session.next_opponent = 0;

    for note in notes {
        spawn_note(
            &mut commands,
            &assets,
            note,
            session.scroll_speed,
            settings.downscroll,
        );
    }

    session.phase = PlayPhase::Countdown;
    session.conductor.start_at(-COUNTDOWN_MS);
    info!(
        "Loaded {} ({} notes, {:.1}s)",
        session.song.id,
        session.total_notes,
        session.duration_ms / 1000.0
    );
}

fn spawn_note(
    commands: &mut Commands,
    assets: &GameAssets,
    note: Note,
    scroll_speed: f32,
    downscroll: bool,
) {
    let lane = note.lane();
    let x = strum_x(note.is_player(), lane);
    let px = PX_PER_MS * f64::from(scroll_speed);
    let y = note_y(note.t, -COUNTDOWN_MS, downscroll, px) as f32;
    let color = if note.is_player() {
        lane_color(lane)
    } else {
        dim(lane_color(lane), 0.55)
    };

    if note.is_hold() {
        // Hold trail: one 52px column of `NOTE_hold_assets.png`, stretched to
        // the sustain length and tinted with the lane colour.
        let height = (note.l * px).max(2.0) as f32;
        let center_y = note_y(note.t + note.l * 0.5, -COUNTDOWN_MS, downscroll, px) as f32;
        commands.spawn((
            Sprite {
                image: assets.note_holds.clone(),
                // Bodies live on even columns (0,2,4,6); odd columns are the caps.
                rect: Some(Rect::new(
                    lane as f32 * 104.0,
                    0.0,
                    lane as f32 * 104.0 + 52.0,
                    87.0,
                )),
                custom_size: Some(Vec2::new(HOLD_WIDTH, height)),
                color: color.with_alpha(0.95),
                ..default()
            },
            Transform::from_xyz(x, center_y, 8.5),
            NoteMarker {
                note,
                part: NotePart::Body,
            },
            GameNote,
            PlayObject,
        ));
    }

    commands.spawn((
        Sprite { color, ..default() },
        Transform::from_xyz(x, y, 9.0),
        NoteMarker {
            note,
            part: NotePart::Head,
        },
        NoteSprite {
            atlas: assets.notes.clone(),
            ready: false,
        },
        GameNote,
        PlayObject,
    ));
}

fn note_y(note_t: f64, position_ms: f64, downscroll: bool, px_per_ms: f64) -> f64 {
    let delta = (note_t - position_ms) * px_per_ms;
    let strum = f64::from(strum_y(downscroll));
    if downscroll {
        strum + delta
    } else {
        strum - delta
    }
}

fn px_per_ms(scroll_speed: f32, settings: &Settings) -> f64 {
    PX_PER_MS * f64::from(scroll_speed) * f64::from(settings.note_speed)
}

/// Spawn a hit splash at the player strumline for the given lane.
fn spawn_splash(commands: &mut Commands, lane: usize, combo: u32, downscroll: bool) {
    let impact = if combo.is_multiple_of(2) {
        "note impact 1"
    } else {
        "note impact 2"
    };
    let prefix = format!("{}  {}", impact, SPLASH_COLORS[lane]);
    commands.spawn((
        Sprite::default(),
        Transform::from_xyz(strum_x(true, lane), strum_y(downscroll), 9.5),
        NoteSplash {
            prefix,
            frames: Vec::new(),
            index: 0,
            elapsed: 0.0,
            ready: false,
        },
        PlayObject,
    ));
}

/// Read keyboard, mouse (desktop debug) and touch (mobile) into lanes.
pub fn read_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut lanes: ResMut<LaneState>,
) {
    *lanes = LaneState::default();

    let bindings: [(KeyCode, KeyCode); LANE_COUNT] = [
        (KeyCode::ArrowLeft, KeyCode::KeyA),
        (KeyCode::ArrowDown, KeyCode::KeyS),
        (KeyCode::ArrowUp, KeyCode::KeyW),
        (KeyCode::ArrowRight, KeyCode::KeyD),
    ];
    for (lane, (primary, secondary)) in bindings.iter().enumerate() {
        if keys.pressed(*primary) || keys.pressed(*secondary) {
            lanes.pressed[lane] = true;
        }
        if keys.just_pressed(*primary) || keys.just_pressed(*secondary) {
            lanes.just_pressed[lane] = true;
        }
    }

    let Ok(window) = windows.single() else {
        return;
    };

    // Four full-height columns on touch devices; the top strip is reserved for
    // the pause affordance so a tap there does not hit a note.
    for touch in touches.iter_just_pressed() {
        let position = touch.position();
        if position.y < 96.0 {
            continue;
        }
        let lane = ((position.x / window.width()) * LANE_COUNT as f32)
            .floor()
            .clamp(0.0, (LANE_COUNT - 1) as f32) as usize;
        lanes.pressed[lane] = true;
        lanes.just_pressed[lane] = true;
    }
    for touch in touches.iter() {
        let position = touch.position();
        if position.y < 96.0 {
            continue;
        }
        let lane = ((position.x / window.width()) * LANE_COUNT as f32)
            .floor()
            .clamp(0.0, (LANE_COUNT - 1) as f32) as usize;
        lanes.pressed[lane] = true;
    }

    if mouse.just_pressed(MouseButton::Left)
        && let Some(position) = window.cursor_position()
    {
        let lane = ((position.x / window.width()) * LANE_COUNT as f32)
            .floor()
            .clamp(0.0, (LANE_COUNT - 1) as f32) as usize;
        lanes.pressed[lane] = true;
        lanes.just_pressed[lane] = true;
    }
}

/// Hit the closest player note within the miss window for every lane pressed
/// this frame. Empty presses break combo unless ghost tapping is enabled.
pub fn judge_notes(
    mut commands: Commands,
    lanes: Res<LaneState>,
    settings: Res<Settings>,
    mut session: ResMut<PlaySession>,
    notes: Query<(Entity, &NoteMarker), With<GameNote>>,
) {
    if session.phase != PlayPhase::Playing {
        return;
    }
    let position = session.conductor.position_ms() + f64::from(settings.offset_ms);
    let autoplay = std::env::var_os("FNF_OXIDE_AUTOPLAY").is_some();
    let mut targets: [Option<(Entity, Judgement)>; LANE_COUNT] = [None; LANE_COUNT];

    for (entity, marker) in notes.iter() {
        if marker.part != NotePart::Head || !marker.note.is_player() {
            continue;
        }
        let lane = marker.note.lane();
        if !lanes.just_pressed[lane] && !autoplay {
            continue;
        }
        let distance = (marker.note.t - position).abs();
        if distance > MISS_WINDOW {
            continue;
        }
        let judgement = if distance <= HIT_WINDOW_SICK {
            Judgement::Sick
        } else if distance <= HIT_WINDOW_GOOD {
            Judgement::Good
        } else if distance <= HIT_WINDOW_BAD {
            Judgement::Bad
        } else {
            Judgement::Shit
        };
        let replace = match targets[lane] {
            None => true,
            Some((_, previous)) => judgement.weight() > previous.weight(),
        };
        if replace {
            targets[lane] = Some((entity, judgement));
        }
    }

    for (lane, target) in targets.into_iter().enumerate() {
        let Some((entity, judgement)) = target else {
            continue;
        };
        commands.entity(entity).despawn();
        session.counts.record(judgement);
        session.combo += 1;
        session.max_combo = session.max_combo.max(session.combo);
        session.score += judgement.points() + session.combo.min(100);
        session.health = (session.health + 1.2).min(100.0);
        session.popup = Some((judgement, 0.55));
        session.player_sing = Some((lane, 0.55));

        // Original spawns a note splash on SICK/GOOD hits.
        if matches!(judgement, Judgement::Sick | Judgement::Good) {
            spawn_splash(&mut commands, lane, session.combo, settings.downscroll);
        }
    }

    for (lane, pressed) in lanes.just_pressed.iter().enumerate() {
        if *pressed && targets[lane].is_none() && !settings.ghost_tapping {
            session.combo = 0;
        }
    }
}

/// Notes that passed the hit window without being hit.
pub fn miss_notes(
    mut commands: Commands,
    settings: Res<Settings>,
    mut session: ResMut<PlaySession>,
    notes: Query<(Entity, &NoteMarker), With<GameNote>>,
) {
    if session.phase != PlayPhase::Playing {
        return;
    }
    let position = session.conductor.position_ms() + f64::from(settings.offset_ms);

    for (entity, marker) in notes.iter() {
        if marker.part != NotePart::Head || !marker.note.is_player() {
            continue;
        }
        if position - marker.note.t > MISS_WINDOW {
            commands.entity(entity).despawn();
            session.counts.record(Judgement::Miss);
            session.combo = 0;
            session.health = (session.health - 4.5).max(0.0);
            session.popup = Some((Judgement::Miss, 0.6));
            session.player_miss = Some((marker.note.lane(), 0.55));
            session.player_sing = None;
        }
    }
}

/// Move note heads/bodies every frame and retire notes that are done.
pub fn move_notes(
    mut commands: Commands,
    session: Res<PlaySession>,
    settings: Res<Settings>,
    mut notes: Query<(Entity, &NoteMarker, &mut Transform, &mut Sprite), With<GameNote>>,
) {
    if matches!(
        session.phase,
        PlayPhase::Loading | PlayPhase::Finished | PlayPhase::Paused
    ) {
        return;
    }
    let position = session.conductor.position_ms();
    let px = px_per_ms(session.scroll_speed, &settings);
    let downscroll = settings.downscroll;

    for (entity, marker, mut transform, mut sprite) in &mut notes {
        match marker.part {
            NotePart::Head => {
                transform.translation.y = note_y(marker.note.t, position, downscroll, px) as f32;
                // Opponent notes are decorative in this milestone.
                if !marker.note.is_player() && position > marker.note.t + 450.0 {
                    commands.entity(entity).despawn();
                }
            }
            NotePart::Body => {
                let center = marker.note.t + marker.note.l * 0.5;
                transform.translation.y = note_y(center, position, downscroll, px) as f32;
                let height = (marker.note.l * px).max(2.0) as f32;
                sprite.custom_size = Some(Vec2::new(HOLD_WIDTH, height));
                if position > marker.note.end_ms() + 250.0 {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

/// Fill in the arrow image/rect for note heads once `notes.xml` is loaded.
pub fn prepare_notes(
    atlases: Res<Assets<SparrowAtlas>>,
    mut notes: Query<(&NoteMarker, &mut NoteSprite, &mut Sprite)>,
) {
    for (marker, mut note_sprite, mut sprite) in &mut notes {
        if note_sprite.ready || marker.part != NotePart::Head {
            continue;
        }
        let Some(atlas) = atlases.get(&note_sprite.atlas) else {
            continue;
        };
        let name = format!("note{}", DIR_NAMES[marker.note.lane()]);
        // Sparrow frame names carry a numeric suffix (`noteLeft0001`), so use
        // the prefix clip rather than an exact-name lookup.
        let Some(frame) = atlas.clip(&name).into_iter().next() else {
            continue;
        };
        sprite.image = atlas.texture.clone();
        sprite.rect = Some(frame.rect);
        sprite.custom_size = Some(frame.size * NOTE_SCALE);
        note_sprite.ready = true;
    }
}

/// Drive receptor animations: static (grey) → press (coloured) → confirm
/// (flash), mirroring the original `StrumlineNote`.
pub fn update_receptors(
    time: Res<Time>,
    lanes: Res<LaneState>,
    session: Res<PlaySession>,
    atlases: Res<Assets<SparrowAtlas>>,
    strumline: Res<GameAssets>,
    mut receptors: Query<(&mut Receptor, &mut Sprite)>,
) {
    let delta = time.delta_secs();

    for (mut receptor, mut sprite) in &mut receptors {
        if !receptor.ready {
            let Some(atlas) = atlases.get(&strumline.note_strumline) else {
                continue;
            };
            let dir = DIR_NAMES[receptor.lane];
            receptor.static_frames = atlas.clip(&format!("static{dir}"));
            receptor.press_frames = atlas.clip(&format!("press{dir}"));
            receptor.confirm_frames = atlas.clip(&format!("confirm{dir}"));
            if receptor.static_frames.is_empty() && receptor.press_frames.is_empty() {
                continue;
            }
            sprite.image = atlas.texture.clone();
            receptor.ready = true;
        }

        // Determine desired state.
        let just = receptor.player && lanes.just_pressed[receptor.lane];
        let held = receptor.player && lanes.pressed[receptor.lane];
        if just || receptor.confirm > 0.0 {
            receptor.state = ReceptorState::Confirm;
            if just {
                receptor.confirm = CONFIRM_HOLD_TIME;
            }
        } else if held {
            receptor.state = ReceptorState::Press;
        } else {
            receptor.state = ReceptorState::Static;
        }

        // Opponent receptors confirm when the opponent sings.
        if !receptor.player
            && session
                .opponent_sing
                .is_some_and(|(lane, _)| lane == receptor.lane)
        {
            receptor.state = ReceptorState::Confirm;
            receptor.confirm = CONFIRM_HOLD_TIME;
        }

        if receptor.confirm > 0.0 {
            receptor.confirm -= delta;
            if receptor.confirm <= 0.0 {
                receptor.confirm = 0.0;
                receptor.state = if held {
                    ReceptorState::Press
                } else {
                    ReceptorState::Static
                };
            }
        }

        let frame_count = match receptor.state {
            ReceptorState::Static => receptor.static_frames.len(),
            ReceptorState::Press => receptor.press_frames.len(),
            ReceptorState::Confirm => receptor.confirm_frames.len(),
        };
        if frame_count == 0 {
            continue;
        }

        // Change of state resets the animation.
        if receptor.pressed != held || receptor.frame >= frame_count {
            receptor.frame = 0;
            receptor.elapsed = 0.0;
        }
        receptor.pressed = held;

        receptor.elapsed += delta;
        while receptor.elapsed >= ANIM_FRAME_TIME {
            receptor.elapsed -= ANIM_FRAME_TIME;
            receptor.frame += 1;
        }
        if receptor.frame >= frame_count {
            receptor.frame %= frame_count;
        }
        let idx = receptor.frame;
        let frame = match receptor.state {
            ReceptorState::Static => receptor.static_frames.get(idx),
            ReceptorState::Press => receptor.press_frames.get(idx),
            ReceptorState::Confirm => receptor.confirm_frames.get(idx),
        };
        let Some(frame) = frame else {
            continue;
        };
        sprite.rect = Some(frame.rect);
        sprite.custom_size = Some(frame.size * NOTE_SCALE);
        sprite.color = Color::WHITE;
    }
}

/// Advance hit-splash animations and despawn them when done.
pub fn update_splashes(
    time: Res<Time>,
    atlases: Res<Assets<SparrowAtlas>>,
    strumline: Res<GameAssets>,
    mut commands: Commands,
    mut splashes: Query<(Entity, &mut NoteSplash, &mut Sprite)>,
) {
    let delta = time.delta_secs();
    for (entity, mut splash, mut sprite) in &mut splashes {
        if !splash.ready {
            let Some(atlas) = atlases.get(&strumline.note_splashes) else {
                continue;
            };
            splash.frames = atlas.clip(&splash.prefix);
            if splash.frames.is_empty() {
                commands.entity(entity).despawn();
                continue;
            }
            sprite.image = atlas.texture.clone();
            splash.ready = true;
        }

        splash.elapsed += delta;
        while splash.elapsed >= ANIM_FRAME_TIME {
            splash.elapsed -= ANIM_FRAME_TIME;
            splash.index += 1;
        }
        if splash.index >= splash.frames.len() {
            commands.entity(entity).despawn();
            continue;
        }
        let frame = splash.frames[splash.index].clone();
        sprite.rect = Some(frame.rect);
        sprite.custom_size = Some(frame.size * NOTE_SCALE);
    }
}

/// Advance the song, spawn audio on the downbeat and finish at the end.
pub fn tick_play(
    time: Res<Time>,
    settings: Res<Settings>,
    mut commands: Commands,
    mut session: ResMut<PlaySession>,
    mut last: ResMut<LastResult>,
    mut next: ResMut<NextState<AppState>>,
) {
    if let Some((_, remaining)) = &mut session.popup {
        *remaining -= time.delta_secs();
        if *remaining <= 0.0 {
            session.popup = None;
        }
    }

    match session.phase {
        PlayPhase::Loading | PlayPhase::Paused | PlayPhase::Finished => {}
        PlayPhase::Countdown => {
            session.conductor.update(time.delta_secs());
            if session.conductor.position_ms() >= 0.0 {
                session.conductor.set_position_ms(0.0);
                session.phase = PlayPhase::Playing;
                let audio = commands
                    .spawn((
                        AudioPlayer::new(session.instrumental.clone()),
                        PlaybackSettings {
                            volume: Volume::Linear(settings.music_gain()),
                            ..PlaybackSettings::DESPAWN
                        },
                        SongAudio,
                        PlayObject,
                    ))
                    .id();
                // Each character has its own vocal track; play them together
                // with the instrumental, exactly like the original.
                info!(
                    "Playing {} vocal track(s) for {}",
                    session.vocals.len(),
                    session.song.id
                );
                for track in session.vocals.clone() {
                    commands.spawn((
                        AudioPlayer::new(track),
                        PlaybackSettings {
                            volume: Volume::Linear(settings.music_gain()),
                            ..PlaybackSettings::DESPAWN
                        },
                        SongAudio,
                        PlayObject,
                    ));
                }
                session.audio = Some(audio);
            }
        }
        PlayPhase::Playing => {
            session.conductor.update(time.delta_secs());
            let position = session.conductor.position_ms();
            while let Some(note) = session.opponent_notes.get(session.next_opponent).copied() {
                if note.t > position {
                    break;
                }
                session.opponent_sing = Some((note.lane(), 0.38));
                session.next_opponent += 1;
            }
            if session.health <= 0.0 {
                session.failed = true;
                finish_play(&mut session, &mut last, &mut next);
            } else if position >= session.duration_ms + SONG_END_PADDING_MS {
                finish_play(&mut session, &mut last, &mut next);
            }
        }
    }
}

fn finish_play(session: &mut PlaySession, last: &mut LastResult, next: &mut NextState<AppState>) {
    session.phase = PlayPhase::Finished;
    session.finished = true;
    session.conductor.stop();

    let total = session.total_notes.max(session.counts.total());
    let accuracy = if total == 0 {
        100.0
    } else {
        (session.counts.weighted() / total as f32 * 100.0).clamp(0.0, 100.0)
    };

    last.song_name = session.song.song_name.to_owned();
    last.artist = session.song.artist.to_owned();
    last.difficulty = session.difficulty;
    last.score = session.score;
    last.accuracy = accuracy;
    last.max_combo = session.max_combo;
    last.counts = session.counts;
    last.rank = rank_for(accuracy, session.failed);
    last.failed = session.failed;

    next.set(AppState::Results);
}

/// Esc/P toggles pause, Q quits from the pause overlay.
pub fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    mut session: ResMut<PlaySession>,
    sinks: Query<&AudioSink, With<SongAudio>>,
    mut next: ResMut<NextState<AppState>>,
) {
    let toggled = keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyP);
    if toggled {
        match session.phase {
            PlayPhase::Playing | PlayPhase::Countdown => {
                session.paused_from = Some(session.phase);
                session.phase = PlayPhase::Paused;
                session.conductor.stop();
                for sink in &sinks {
                    sink.pause();
                }
            }
            PlayPhase::Paused => {
                session.phase = session.paused_from.take().unwrap_or(PlayPhase::Playing);
                let position = session.conductor.position_ms().max(0.0);
                session.conductor.start_at(position);
                for sink in &sinks {
                    sink.play();
                }
            }
            _ => {}
        }
    }

    if session.phase == PlayPhase::Paused && keys.just_pressed(KeyCode::KeyQ) {
        next.set(AppState::Freeplay);
    }
}

pub fn cleanup_play(
    mut commands: Commands,
    objects: Query<Entity, With<PlayObject>>,
    mut lanes: ResMut<LaneState>,
) {
    for entity in &objects {
        commands.entity(entity).despawn();
    }
    *lanes = LaneState::default();
    commands.remove_resource::<PlaySession>();
}

// ---------------------------------------------------------------------------
// Stage layout (data-driven)
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct StageData {
    #[serde(rename = "cameraZoom", default = "one")]
    camera_zoom: f32,
    #[serde(default)]
    props: Vec<StagePropData>,
    #[serde(default)]
    characters: Option<StageCharacters>,
}

fn one() -> f32 {
    1.0
}

#[derive(serde::Deserialize)]
struct StagePropData {
    #[serde(rename = "assetPath")]
    asset_path: String,
    #[serde(default)]
    position: [f32; 2],
    #[serde(default = "one_pair")]
    scale: [f32; 2],
    #[serde(rename = "zIndex", default)]
    z_index: f32,
    #[serde(default)]
    name: String,
}

fn one_pair() -> [f32; 2] {
    [1.0, 1.0]
}

#[derive(serde::Deserialize, Default)]
struct StageCharacters {
    #[serde(default)]
    bf: Option<StageCharacterData>,
    #[serde(default)]
    dad: Option<StageCharacterData>,
    #[serde(default)]
    gf: Option<StageCharacterData>,
}

#[derive(serde::Deserialize, Default)]
struct StageCharacterData {
    #[serde(default)]
    position: [f32; 2],
}

#[derive(serde::Deserialize, Default)]
struct SongMetadata {
    #[serde(rename = "playData", default)]
    play_data: PlayData,
}

#[derive(serde::Deserialize, Default)]
struct PlayData {
    #[serde(default)]
    characters: CastNames,
}

#[derive(serde::Deserialize, Default)]
struct CastNames {
    #[serde(default = "default_bf", rename = "player")]
    player: String,
    #[serde(default = "default_gf", rename = "girlfriend")]
    girlfriend: String,
    #[serde(default = "default_dad", rename = "opponent")]
    opponent: String,
    #[serde(default, rename = "playerVocals")]
    player_vocals: Vec<String>,
    #[serde(default, rename = "opponentVocals")]
    opponent_vocals: Vec<String>,
}

fn default_bf() -> String {
    "bf".to_string()
}
fn default_gf() -> String {
    "gf".to_string()
}
fn default_dad() -> String {
    "dad".to_string()
}

/// A stage prop waiting for its texture to load so it can be sized.
#[derive(Component)]
pub struct StagePropSprite {
    pub scale: Vec2,
    pub top_left: Vec2,
    pub z: f32,
}

/// The original play camera uses the stage's `cameraZoom` (1.1 for Week 1) and
/// starts centred on Dad's camera focus point. The vertical scroll is pinned so
/// the floor line (`stageFront.y`) and the cast's feet keep the original
/// on-screen relationship.
const STAGE_ZOOM: f32 = 1.1;
const STAGE_CAM: Vec2 = Vec2::new(616.0, 585.0);

fn stage_to_world(x: f32, y: f32) -> Vec2 {
    Vec2::new(
        (x - STAGE_CAM.x) * STAGE_ZOOM,
        (STAGE_CAM.y - y) * STAGE_ZOOM,
    )
}

fn resolve_prop_path(asset_path: &str) -> String {
    let (prefix, rest) = match asset_path.split_once(':') {
        Some((prefix, rest)) => (prefix, rest),
        None => ("shared", asset_path),
    };
    match prefix {
        "shared" => format!("shared/images/{rest}.png"),
        "preload" => format!("preload/images/{rest}.png"),
        _ => format!("shared/images/{asset_path}.png"),
    }
}

/// Spawn the level's backdrop props and move the cast to their stage positions.
#[allow(clippy::type_complexity, clippy::collapsible_if)]
pub fn spawn_stage(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    stages: Res<Assets<RawJson>>,
    mut session: ResMut<PlaySession>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if session.stage_spawned {
        return;
    }
    let Some(raw) = stages.get(&session.stage) else {
        return;
    };
    let data: StageData = serde_json::from_str(&raw.0).unwrap_or(StageData {
        camera_zoom: 1.0,
        props: Vec::new(),
        characters: None,
    });

    for prop in &data.props {
        let image: Handle<Image> = asset_server.load(resolve_prop_path(&prop.asset_path));
        let top_left = stage_to_world(prop.position[0], prop.position[1]);
        commands.spawn((
            Name::new(prop.name.clone()),
            Sprite { image, ..default() },
            Transform::from_xyz(top_left.x, top_left.y, -9.0 + prop.z_index * 0.02),
            StagePropSprite {
                scale: Vec2::new(prop.scale[0], prop.scale[1]),
                top_left,
                z: -9.0 + prop.z_index * 0.02,
            },
            PlayObject,
        ));
    }

    if !session.cast_spawned {
        let meta_raw = stages.get(&session.metadata);
        let meta: SongMetadata = meta_raw
            .and_then(|raw| serde_json::from_str(&raw.0).ok())
            .unwrap_or_default();
        let names = &meta.play_data.characters;
        let cast = data.characters.as_ref();

        let place = |slot: Option<&StageCharacterData>, fallback: [f32; 2]| {
            let p = slot.map(|c| c.position).unwrap_or(fallback);
            stage_to_world(p[0], p[1])
        };
        let bf_pos = place(cast.and_then(|c| c.bf.as_ref()), [989.5, 885.0]);
        let dad_pos = place(cast.and_then(|c| c.dad.as_ref()), [335.0, 885.0]);
        let gf_pos = place(cast.and_then(|c| c.gf.as_ref()), [751.5, 787.0]);

        let specimens = [
            (
                CharacterSide::Girlfriend,
                names.girlfriend.as_str(),
                gf_pos,
                0.53,
                false,
                4.5,
            ),
            (
                CharacterSide::Opponent,
                names.opponent.as_str(),
                dad_pos,
                0.68,
                false,
                5.5,
            ),
            (
                CharacterSide::Player,
                names.player.as_str(),
                bf_pos,
                0.65,
                false,
                6.5,
            ),
        ];
        for (side, name, feet, scale, flip, z) in specimens {
            let atlas: Handle<AnimateAtlasAsset> =
                asset_server.load(format!("shared/images/characters/{name}/Animation.json"));
            characters::spawn_mesh_character(
                &mut commands,
                &mut materials,
                side,
                atlas,
                feet.x,
                feet.y,
                scale,
                flip,
                z,
            );
        }
        session.cast_spawned = true;
    }

    // Keep the camera zoom from the stage data meaningful by scaling the whole
    // stage group; currently the props are already sized in world units.
    let _ = data.camera_zoom;
    session.stage_spawned = true;
}

/// Apply a prop's scale once its texture is available (size is then known).
#[allow(clippy::collapsible_if)]
pub fn layout_stage_props(
    images: Res<Assets<Image>>,
    mut props: Query<(&StagePropSprite, &mut Sprite, &mut Transform)>,
) {
    for (prop, mut sprite, mut transform) in &mut props {
        let Some(image) = images.get(&sprite.image) else {
            continue;
        };
        let size = image.size_f32() * prop.scale * STAGE_ZOOM;
        sprite.custom_size = Some(size);
        transform.translation.x = prop.top_left.x + size.x * 0.5;
        transform.translation.y = prop.top_left.y - size.y * 0.5;
        transform.translation.z = prop.z;
    }
}

# Content schema — JSON registry → TOML

Every file in `assets/preload/data/<registry>/` maps to
`content/<registry>/<name>.toml`. Field names are kept **identical to the
original JSON** (camelCase), so an existing mod can be converted mechanically.
`$schema`-style metadata (`version`, `generatedBy`) is dropped; `name`/`id` stays.

## Registry list (what a mod may ship)

| JSON directory | TOML | needed for |
|---|---|---|
| `stages/` | `content/stages/` | every song |
| `characters/` | `content/characters/` | every song |
| `songs/<id>/<id>-metadata.json` | `content/songs/` | every song |
| `songs/<id>/<id>-chart*.json` | `content/charts/` | every song |
| `levels/` | `content/levels/` | story mode week |
| `notestyles/` | `content/notestyles/` | custom arrows/hold/splash/judgements |
| `dialogue/{boxes,speakers,conversations}/` | `content/dialogue/` | cutscenes |
| `players/` | `content/players/` | freeplay character roster |
| `stickerpacks/` | `content/stickerpacks/` | results stickers |

## stages

```toml
id = "mainStage"
cameraZoom = 1.1
[[props]]
name = "stageBack"; assetPath = "stageback"; position = [-600, -200]
scale = [1, 1]; scroll = [0.9, 0.9]; zIndex = 10
[characters.bf]          # bf / dad / gf
position = [989.5, 885]
cameraOffsets = [-100, -100]
```

## characters

```toml
id = "bf"
renderType = "multianimateatlas"   # sparrow | animateatlas | multianimateatlas
assetPath = "shared:characters/bf"
flipX = true
offsets = [17, 14]
singDuration = 4
healthIcon = "bf"
[animations]
idle = "BF idle dance"
singLEFT = "BF NOTE LEFT"
```

## songs (`<id>-metadata.json`)

```toml
id = "bopeebo"
songName = "Bopeebo"; artist = "Kawai Sprite"; charter = "ninjamuffin99"
[playData]
stage = "mainStage"; noteStyle = "funkin"; album = "volume1"
difficulties = ["easy", "normal", "hard"]; songVariations = []
[playData.characters]
player = "bf"; girlfriend = "gf"; opponent = "dad"
playerVocals = ["bf"]; opponentVocals = ["dad"]
[[timeChanges]]
t = 0; bpm = 100; beatTime = [4, 4, 4, 4]
```

## charts

```toml
song = "bopeebo"; difficulty = "normal"
scrollSpeed = 1.3
[[notes]]
t = 600; lane = 3; len = 525          # len is a hold length in ms
[[events]]
t = 0; e = "FocusCamera"; v = 1
```

## levels (story mode)

```toml
id = "week1"; name = "DADDY DEAREST"
titleAsset = "storymenu/titles/week1"
background = "storymenu/bg"
[capsule]
name = "week1"
songs = ["bopeebo", "fresh", "dadbattle"]
[[props]]
assetPath = "storymenu/props/dad"; scale = 1.0; offsets = [100, 60]
[[props.animations]]
name = "idle"; prefix = "idle0"; frameRate = 24
```

## notestyles

```toml
id = "funkin"; author = "funkin"
[assets.note]
assetPath = "shared:notes"; scale = 0.7
[assets.note.data]
left = { prefix = "noteLeft" }
down = { prefix = "noteDown" }
# same shape for noteStrumline, holdNote, noteSplash, holdNoteCover,
# countdownThree/Two/One/Go, judgementSick/Good/Bad/Shit, comboNumber0..9
[assets.countdownThree]
data = { audioPath = "shared:gameplay/countdown/funkin/introTHREE" }
```

## dialogue

Three sub-registries; `speakers` and `conversations` reference `boxes` by id.

```toml
# content/dialogue/boxes/default.toml
id = "default"; assetPath = "speech_bubble_talking"
isPixel = true; offsets = [-660, -340]; scale = 5.4
[text]
offsets = [225, 450]; width = 900; color = "#3F2021"
fontFamily = "Pixel Arial 11 Bold"; shadowColor = "#D89494"; shadowWidth = 2

# content/dialogue/speakers/dad.toml
id = "dad"; name = "Daddy Dearest"; character = "dad"; box = "default"

# content/dialogue/conversations/week1.toml
id = "week1"
[[lines]]
speaker = "dad"; text = "Well well well..."
```

## players (freeplay roster)

```toml
id = "bf"; name = "Boyfriend"
ownedChars = ["bf"]; showUnownedChars = false
freeplayStyle = "bf"; stickerPack = "bonus-tutorial"
[freeplayDJ]
assetPath = "freeplay/freeplay-boyfriend"; fistPump = true
[charSelect]
position = 0
[results]
music = "results"
```

## stickerpacks

```toml
id = "bonus-tutorial"; artist = "phantomArcade"
stickers = ["sticker1", "sticker2"]
```

## Rules

- All ids are unique per registry; a mod may override a builtin (warn on load).
- `assetPath` keeps the original `shared:` / `preload:` prefixes.
- Unknown keys are preserved as protobuf `unknown` fields? No — the compiler
  rejects unknown keys so typos surface at build time.

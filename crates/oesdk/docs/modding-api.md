# Oxide 模组 API 设计（对照 FNF 0.8.x hxc 文档）

参考：`fnf-hxc-docs/fnf-hxc-script-guide.md`。原版的核心不是「函数表」，而是
**数据 + 脚本化类**：JSON 决定内容，`.hxc` 脚本 `extends` 基类并用
`super('id')` 接管同 id 的数据。Oxide 保留这个心智模型，但把「继承」换成
「实现 trait + 常量 id」，把「解释器」换成 wasm 组件。

## 1. 设计约束

- **不暴露 ECS**。脚本只拿到 `Ctx`：只读查询 + 指令（`PlayAnim`、`SpawnSprite`…），
  指令在帧边界统一应用，避免系统执行顺序泄漏。
- **内容可无脚本**。JSON/TOML 能表达的东西不要求写代码；脚本只加行为。
- **数据与行为分离**。`stages/foo.toml` 描述道具，`FooStage` 脚本描述动作。
- **确定性**。同一帧内事件按 `(priority, mod_id)` 排序；`cancel` 只影响后续默认行为。
- **安全**：无文件系统、无网络、无环境变量；燃料 + epoch 双重预算。

## 2. Haxe → Oxide 映射

| Haxe 基类 | 数据目录 | Oxide 对应 |
| --- | --- | --- |
| `Stage` | `data/stages/<id>.json` | `StageData` (TOML) + `impl StageScript` |
| `StageProp` / `Bopper` | 舞台 JSON 的 `props[]` | `StageProp` 组件，字段 `dance_every` 区分 |
| `Song` | `data/songs/<id>/<id>-metadata.json` | `SongMeta` (TOML→proto) + `impl SongScript` |
| `BaseCharacter` 家族 | `data/characters/<id>.json` | `CharacterData` + `impl CharacterScript` |
| `NoteKind` | 无（谱面 `kind`） | `NoteKind` 注册表 + `impl NoteKindScript` |
| `NoteStyle` | `data/notestyles/<id>.json` | `NoteStyleData` |
| `SongEvent` | 无（谱面 `e`） | `impl SongEventScript`，`EVENT_NAME` 常量 |
| `Level` | `data/levels/<id>.json` | `LevelData` |
| `Module` | 无 | `impl Module`，`MODULE_ID` 常量 |
| `Conversation` 等 | `data/dialogue/**` | 后续阶段 |

脚本类的唯一身份 = **`const ID: &str`**（等于 `super('...')` 的字符串）。
注册表按 `(content_kind, id)` 查表，命中就实例化脚本，否则用默认行为。

## 3. 内容包

```
mymod/
  mod.toml            # 清单（见 3.1）
  content/
    stages/myStage.toml
    characters/myChar.toml
    songs/mySong/song.toml
    songs/mySong/chart.toml
    notestyles/neon.toml
    levels/weekx.toml
  src/lib.rs          # 脚本，编译成 wasm 组件
  assets/             # 覆盖同路径资源（仅限本 mod 命名空间）
```

### 3.1 `mod.toml`

```toml
id = "com.example.mymod"
name = "My Mod"
version = "1.0.0"
api = "0.2"                 # 引擎 ABI 主版本，semver 匹配
entry = "mymod.wasm"        # 可选：纯数据 mod 可省略
capabilities = ["content", "script"]
authors = ["you"]
```

### 3.2 舞台 `stages/<id>.toml`

对应原版 `StageData`：

```toml
id = "myStage"
camera_zoom = 1.1
camera_focus = [616.0, 585.0]

[[props]]
name = "back"
asset = "shared:stageback"      # 同原版 assetPath 前缀规则
position = [-600.0, -200.0]
scale = [1.0, 1.0]
scroll = [0.9, 0.9]
z = 10
dance_every = 0                  # >0 时成为 Bopper
animations = []                  # 可选：Sparrow 前缀列表

[characters.bf]
position = [989.5, 885.0]
camera_offsets = [-100.0, -100.0]
[characters.dad]
position = [335.0, 885.0]
[characters.gf]
position = [751.5, 787.0]
```

`dance_every > 0` 或 `animations` 非空 → 构建为 `Bopper`（随节拍播放动画），
否则是静态 `StageProp`。`z` 排序语义与原版一致（小在前，角色 100/200/300）。

### 3.3 角色 `characters/<id>.toml`

```toml
id = "myChar"
name = "My Character"
render_type = "animate"          # sparrow | animate | multi_animate
asset = "shared:characters/myChar"
scale = 1.0
flip_x = false
offsets = [0.0, 0.0]
sing_duration = 0.35             # 原版 4/24 秒
health_icon = "icon-myChar"
death_quote = "..."

[animations]
idle = ["Idle"]
sing_left = ["Left", "singLEFT"]
sing_down = ["Down"]
sing_up = ["Up"]
sing_right = ["Right"]
miss_left = ["Left Miss"]
```

渲染器按 `render_type` 选择加载器：`sparrow` → `SparrowAtlas`，
`animate`/`multi_animate` → `AnimateAtlasAsset`（mesh 路径）。

### 3.4 曲谱 `songs/<id>/song.toml` + `chart.toml`

`SongMeta` 沿用现有 `-metadata.json` 字段（`play_data.characters`、`time_changes`、
`difficulties`、`variations`），TOML 只是书写层；构建期转 proto。
`chart.toml` 的 `notes` 允许 `kind` 字段（默认 `"normal"`），`events` 的 `e`
可以是内置名或 mod 注册的 `SongEventScript::EVENT_NAME`：

```toml
[[notes.normal]]
t = 600.0
d = 7
l = 525.0
kind = "ghost"

[[events]]
t = 4200.0
e = "MyLights"
v = { color = "red", duration = 0.5 }
```

## 4. 脚本 trait

```rust
pub trait StageScript: Sized {
    const ID: &'static str;                 // = super('id')
    fn new(data: &StageData) -> Self;
    fn on_create(&mut self, _ctx: &mut Ctx) {}
    fn on_update(&mut self, _ctx: &mut Ctx) {}
    fn on_beat(&mut self, _ctx: &mut Ctx, _beat: u32) {}
    fn on_step(&mut self, _ctx: &mut Ctx, _step: u32) {}
    fn on_note_hit(&mut self, _ctx: &mut Ctx, _hit: &Hit) {}
    fn on_note_miss(&mut self, _ctx: &mut Ctx, _miss: &Miss) {}
    fn on_song_event(&mut self, _ctx: &mut Ctx, _event: &SongEventData) {}
    fn on_destroy(&mut self, _ctx: &mut Ctx) {}
}
```

`CharacterScript` 对应 `BaseCharacter`（`play_sing`, `dance`, `health_icon`,
`death_*`），`SongScript` 对应 `Song`（`on_song_loaded`、`on_song_start`、
`chart_mut`），`Module` 没有数据 id，只有一个 `MODULE_ID` 与全局生命周期：

```rust
pub trait Module {
    const MODULE_ID: &'static str;
    fn on_create(&mut self, _ctx: &mut Ctx) {}
    fn on_script_event(&mut self, _ctx: &mut Ctx, _ev: &ScriptEvent) {}
    fn on_state_change(&mut self, _ctx: &mut Ctx, _from: StateId, _to: StateId) {}
    fn on_destroy(&mut self, _ctx: &mut Ctx) {}
    fn needs_update(&self) -> bool { false }
}
```

`Module` 常驻，不随 PlayState 销毁；`needs_update=false` 时引擎不每帧调用，
只派发事件（省电，不是性能必需）。

## 5. 事件与顺序

| 事件 | 时机 | 可取消 |
| --- | --- | --- |
| `on_create` | 内容实例化后 | 否 |
| `on_song_loaded` | 谱面/元数据就绪 | 是（改谱面） |
| `on_countdown_start/step/end` | 倒计时 | 否 |
| `on_song_start` | 第一拍 | 否 |
| `on_beat` / `on_step` | 传导器节拍 | 否 |
| `on_note_incoming` | 音符进入判定线前 | 是（改速度/隐藏） |
| `on_note_hit` / `on_note_miss` / `on_hold_drop` / `on_ghost_miss` | 判定 | 是（改分数/特效） |
| `on_song_event` | 谱面 `events` | 是 |
| `on_pause` / `on_resume` | 暂停 | 否 |
| `on_song_end` / `on_game_over` / `on_song_retry` | 结束 | 否 |
| `on_destroy` | 退出 | 否 |

分派：按 `priority` 升序，同优先级按 `mod id` 字典序；`event.cancel()` 停止
后续默认行为，`event.stop_propagation()` 只停止后续 mod。两者都不影响已经
执行过的脚本。

## 6. 注册表与覆盖

- 启动时扫描 `mods/*/mod.toml`（Android：应用私有目录；桌面：`./mods`）。
- 内容注册表分 kind：`Stage/Character/Song/NoteStyle/NoteKind/Level/SongEvent/Module`。
- 同 id 冲突：**mod 覆盖 builtin** 并 `warn!`；两个 mod 冲突则按加载顺序后者生效，
  同时记录诊断（`oesdk doctor` 可列出）。
- `api` 与引擎 ABI 主版本不一致 → 拒绝加载该 mod，不影响其他 mod。
- 热重载：`F5` 卸载实例 → 重新扫描 → 重建；脚本静态状态丢失（与原版一致）。

## 7. 与现有 ABI 的衔接

`wit/oesdk.wit` 增加四个 world-level export（`stage`、`character`、`song`、`module`），
每个 export 带 `id()` 与生命周期函数；`content` import 提供只读查询
（`stage-prop(id)`、`character(id)`、`note-style(id)`）。现有 `Ctx` 的
`character/cast/camera/rng/sprite` 保持不变。

## 8. 分期

1. **P1 数据层**：TOML→proto，`StageData`/`CharacterData`/`SongMeta` 运行时注册，
   无脚本也能加载自定义舞台/人物/曲谱（最大收益，先做）。
2. **P2 行为层**：`StageScript`/`CharacterScript`/`SongScript` 三个 trait 与
   `on_note_hit` 等 8 个高频事件。
3. **P3 Module**：常驻模块 + `on_script_event` + 状态切换。
4. **P4**：`NoteKind`/`NoteStyle`/`Level`/对话/自定义 State。


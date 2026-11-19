# FNF: Oxide Engine

用 **Rust + Bevy 0.20** 重构 *Friday Night Funkin'* 的引擎骨架。

当前阶段的目标是**能跑通的骨架**：

- 桌面端可启动、可完整走通 `Boot → Title → MainMenu → StoryMenu/Freeplay → Play → Results`
- 读取原版 `assets/` 中的 JSON 谱面与 `.ogg` 音频，27 首内置歌曲
- 4 lane 下落式玩法、长条、判定、连击、分数、准确率、血条、暂停
- 还原 Week 1 舞台（stageback / curtains / stagefront）、BF/Dad/GF 角色精灵、标题界面 GF 舞蹈
- Story Mode 周目选择（10 个周目、难度切换、角色剪影、左右箭头）
- Freeplay 按原版布局重做：`◀ NORM ▶` 难度选择、纵向胶囊歌曲列表、HIGHSCORE、OST 专辑封面
- Play 受体左右对称、原版 notes/noteStrumline 图集、press/confirm 受体动画与 note splash
- Options 可调音量 / Downscroll / Note Speed / Input Offset / Ghost Tapping，并持久化到 JSON
- **Android** Gradle + GameActivity 工程骨架
- **iOS** Xcode 工程骨架（Bevy 新版“Xcode 只做壳、Cargo 产出可执行文件”方案）

## 截图

| Title | Freeplay |
| --- | --- |
| ![Title](docs/screenshots/shot-title.png) | ![Freeplay](docs/screenshots/shot-freeplay.png) |

| Play | Story Mode |
| --- | --- |
| ![Play](docs/screenshots/shot-play.png) | ![Story Mode](docs/screenshots/shot-storymenu.png) |

| Main Menu | Options |
| --- | --- |
| ![Main Menu](docs/screenshots/shot-mainmenu.png) | ![Options](docs/screenshots/shot-options.png) |

> 角色使用原版 Sparrow 图集；Dad 的 Adobe Animate 图集由 `AnimateLoader` 在启动时烘焙为 Sparrow。
> 更多截图见 [`docs/screenshots`](docs/screenshots)。

---

## 快速开始（桌面）

```bash
cd rust/FNF-Oxide-Engine
cargo run --bin OxideEngine
```

首次编译 Bevy 需要几分钟。需要 Rust 1.97+（当前使用 1.99）。

### 操作

| 场景 | 按键 |
| --- | --- |
| Title | `Enter` / `Space` / 鼠标左键 |
| Main Menu | `F` Freeplay，`O` Options，`Esc` 返回 |
| StoryMenu | 左右切换周目，上下切难度，`Enter` 开始，`Esc` 返回 |
| Freeplay | 方向键选择歌曲，`Q`/`E` 或左右键切难度，`Enter` 开始，`Esc` 返回 |
| Play | `A S W D` 或方向键打歌，`Esc` / `P` 暂停，暂停时 `Q` 退出 |
| Options | `Esc` 返回；鼠标点击 `±` / `TOGGLE` |
| Results | `Enter` 重试，`Esc` 回 Freeplay |

触屏：Play 中屏幕底部按四列划分，对应四个 lane；菜单按钮均可点击。

### 资源路径

默认读取项目内 `assets/`（编译期用 `CARGO_MANIFEST_DIR` 定位），可用环境变量覆盖：

> `assets/` 是从 `haxe/funkin/assets` 复制的原版素材（约 1.3 GB），未纳入 git；克隆后请先复制该目录。

```bash
FNF_OXIDE_ASSETS=/path/to/assets cargo run --bin OxideEngine
```

存档默认位于：

- Linux：`$XDG_DATA_HOME/fnf-oxide/settings.json`（否则 `~/.local/share/fnf-oxide/settings.json`）
- macOS：`~/Library/Application Support/fnf-oxide/settings.json`
- Windows：`%APPDATA%/fnf-oxide/settings.json`
- Android：应用 internal data 目录
- iOS：`$HOME/Documents/fnf-oxide/settings.json`

也可用 `FNF_OXIDE_DATA_DIR` 指定目录。

---

## 工程结构

```
rust/FNF-Oxide-Engine/
├── Cargo.toml                 # workspace + 可执行 crate
├── src/
│   ├── main.rs                # 桌面 / iOS 入口
│   ├── lib.rs                 # run() + Android #[bevy_main]
│   └── app.rs                 # DefaultPlugins、AssetPlugin、状态与插件装配
├── crates/
│   ├── oxide_core/            # 状态机、配置、Settings/Save、Sparrow atlas、主题、Conductor
│   ├── oxide_data/            # 谱面 JSON、曲目 catalog（build.rs 扫描 assets）
│   ├── oxide_game/            # Play 会话、下落/判定/长条、角色动画、HUD、结算数据
│   └── oxide_ui/              # Boot/Title/MainMenu/Freeplay/Options/Results
├── tools/build_visual_assets.py  # 从原版 atlases 生成 Dad 精灵表、UI 切片
├── assets/                    # 原版素材 + 派生素材（oxide/characters、oxide/ui、oxide/notes…）
├── android/                   # Android Studio / Gradle 工程
├── ios/                       # Xcode 工程 + Cargo 构建脚本
└── docs/screenshots/
```

### 核心设计

- **Workspace 分层**：数据格式与玩法/UI 解耦，后续可替换渲染层或加入 mod 加载。
- **状态机**：`AppState { Boot, Title, MainMenu, StoryMenu, Freeplay, Options, Play, Results }`，每个界面在 `OnEnter` 生成、`OnExit` 递归清理。
- **曲目 catalog**：`oxide_data/build.rs` 在编译期扫描
  `assets/preload/data/songs/*/*-metadata.json`，生成 `BUILTIN_SONGS`，因此新增歌曲不需要改 Rust 代码。
- **谱面**：直接反序列化原版 chart v2.0.0（`notes`、`scrollSpeed`、`events`），
  `d % 4` 为 lane，`d >= 4` 为玩家侧；`l` 是长条长度（毫秒）。
- **Conductor**：以毫秒为单位推进歌曲时间，倒计时阶段从 `-2600ms` 开始，音频在时间归零时启动。
  判定使用 `Conductor.position_ms()`，暂停时同步 `AudioSink`。
- **输入**：键盘与触摸统一写入 `LaneState { pressed, just_pressed }`，判定层不关心来源。
- **Sparrow 图集**：`oxide_core::sparrow` 注册自定义 `.xml` asset loader，解析 `SubTexture`
  并保留 trim offset；角色、标题 GF 与菜单素材都通过它加载。
- **角色动画**：`oxide_game::characters` 以 24fps 循环 idle/sing/miss clip，按 conductor beat
  做 idle bob；Dad 的 Animate Atlas 由 `tools/build_visual_assets.py` 预烘焙。

---

## Android

工程：`android/`，包名 `com.oxide.engine`，`MainActivity extends com.google.androidgamesdk.GameActivity`，
通过 Manifest 的 `android.app.lib_name=fnf_oxide_engine` 加载 Rust cdylib。

### 环境

- Android SDK：compileSdk / targetSdk 36，minSdk 26
- NDK：`29.0.14206865`（或修改 `android/app/build.gradle` 中的 `ndkVersion`）
- JDK **17–23**（Gradle 8.13 的运行上限是 23）。JDK 24/25 会在编译构建脚本时报
  `BUG! exception in phase 'semantic analysis' ... Unsupported class file major version 69`，
  因为 69 = Java 25 的 class 文件版本。
- Rust：`rustup target add aarch64-linux-android`
- 可选：`cargo install cargo-ndk`（本工程自带 Gradle → Cargo 任务，不强制）

### 构建

```bash
cd rust/FNF-Oxide-Engine/android
./build.sh :app:assembleDebug
```

`build.sh` 会挑一个 JDK 17–23（优先 `JAVA_HOME`，其次仓库旁的 `.toolchain/jdk21`、
系统 JDK 21/17），并设置 `ANDROID_HOME`、本地 `GRADLE_USER_HOME` / `CARGO_HOME`，
然后转交给 `./gradlew`。想手动调用时：

```bash
export JAVA_HOME=/usr/lib/jvm/java-21-openjdk-amd64   # 必须是 JDK，不能是 JRE
export ANDROID_HOME="$HOME/Android/Sdk"               # 或 local.properties: sdk.dir=...
./gradlew :app:assembleDebug
```

Gradle 会：

1. 运行 `cargo build --release --target aarch64-linux-android --lib`
2. 把 `libfnf_oxide_engine.so` 复制到 `app/src/main/jniLibs/arm64-v8a/`
3. 把仓库根 `assets/` 作为 APK assets 打包

Rust cdylib 已静态链接 C++ 运行时，APK 不需要额外的 `libc++_shared.so`，因此工程不依赖 SDK 里的 CMake。

APK 输出：`android/app/build/outputs/apk/debug/app-debug.apk`

安装：

```bash
adb install -r android/app/build/outputs/apk/debug/app-debug.apk
adb shell am start -n com.oxide.engine/.MainActivity
```

### 说明

- 目前只构建 `arm64-v8a`；模拟器需要额外加入 `x86_64`（同时修改 `abiFilters` 与 Gradle 的 Rust 任务）。
- 若 SDK 目录只读，请把 `ANDROID_USER_HOME` / `GRADLE_USER_HOME` 指到可写目录。
- `assets/` 约 1.3 GB，打包/安装会比较慢，这是原版素材体积导致的。

---

## iOS

工程：`ios/FNF-Oxide-Engine.xcodeproj`，product 为 `OxideEngine.app`，Bundle ID `com.oxide.engine`。

采用 Bevy 新版 iOS 方案（无 `main.m`、无 staticlib）：

- `ios/build_rust_deps.sh` 在 Xcode 的 `Rust` build phase 中执行
  `cargo build --bin OxideEngine --target aarch64-apple-ios`（模拟器对应 `*-apple-ios-sim` / `x86_64-apple-ios`）
- 多架构时用 `lipo -create` 输出到 `$TARGET_BUILD_DIR/$EXECUTABLE_PATH`
- Xcode target 本身没有源码文件，只负责签名、Info.plist 与资源拷贝

### 构建（需要 macOS + Xcode）

```bash
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
open rust/FNF-Oxide-Engine/ios/FNF-Oxide-Engine.xcodeproj
```

在 Xcode 中选择签名 Team 后运行。首次构建会在 `DerivedData` 内执行 Cargo 编译。

### 说明

- 必须在 macOS 上构建；Linux 无法链接 iPhoneOS SDK。
- `assets` 以 folder reference 方式加入 app bundle，运行时路径即 `assets/`，与 `asset_root()` 的移动端分支一致。
- 如果新增 Cargo 依赖并链接系统库，可能需要在 Xcode target 的 *Frameworks, Libraries, and Embedded Content* 中补充（Bevy 默认依赖通常已由 Rust 链接）。

---

## 当前范围与后续

已实现：

- 27 首歌曲的原版 Freeplay 胶囊列表、难度选择、HIGH SCORE 与 OST 专辑
- 基础谱面（`<id>-chart.json`）与 Inst.ogg 播放
- 4 lane 判定（SICK/GOOD/BAD/SHIT/MISS）、长条、连击、分数、准确率、血量
- Week 1 舞台与 BF/Dad/GF 角色精灵、sing/miss 动画、血条角色图标
- 左右对称的 strumline、原版 notes/noteStrumline 图集、受体 static/press/confirm 动画、note splash
- Story Mode 周目选择（10 周、难度切换、角色剪影）
- 标题界面 GF 舞蹈、原版手写标题 logo 与 Freeplay/Options 菜单图
- 左上角 FPS / 进程内存（current / peak）显示（复刻原版 `FunkinDebugDisplay`）
- 暂停/恢复、失败与结算、重试
- Settings 持久化
- 1280×720 设计分辨率，UI 按 `UiScale` 自适应

尚未实现（骨架阶段的已知缺口）：

- 镜头事件（`events` 已解析但未执行）、舞台灯光/物件动画
- GF 的 Animate Atlas 运行时渲染（当前标题/游戏内使用预烘焙 Sparrow）
- 变体谱面（Erect/Pico）与 vocals 音轨
- 歌曲选择预览、Mods、Week 流程
- 移动端安全区/刘海适配与更精细的触控布局
- 音频延迟校准的自动测量

---

## 开发辅助

桌面端支持以下环境变量，便于截图与 UI 回归：

```bash
FNF_OXIDE_START=play \
FNF_OXIDE_AUTOPLAY=1 \
FNF_OXIDE_SHOT_DELAY=7.5 \
FNF_OXIDE_SCREENSHOT=/tmp/play.png \
cargo run --bin OxideEngine
```

- `FNF_OXIDE_START`：`title` / `mainmenu` / `freeplay` / `options` / `play` / `results`
- `FNF_OXIDE_SCREENSHOT`：延迟后保存一张窗口截图（同时禁用 Title 的自动进入）
- `FNF_OXIDE_SHOT_DELAY`：截图延迟秒数，默认 4.0
- `FNF_OXIDE_AUTOPLAY`：在 Play 中自动命中玩家音符，方便拍角色动画/判定截图

这些变量只在桌面端生效，不影响移动端。

## License

代码沿用仓库根目录的 **GNU AGPL v3**（见 `LICENSE`）。`assets/` 中的美术、音乐与谱面来自 *Friday Night Funkin'* 原项目，版权归其各自作者所有；分发或商用前请自行确认原项目与素材的授权条款。

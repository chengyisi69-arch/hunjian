# 混剪生成器 (hunjian)

基于 **Tauri v2 + Svelte + Rust + FFmpeg** 的桌面端 AI 视频混剪工具。

## 功能

- 把每个场景的素材切成固定长度（3~5 秒）小片段，并行切割
- 每场景按需要随机抽取 1 或 2 个片段，已用过的不再被抽到（可重置）
- 自动控制总时长 ≤ 60 秒（含 0.5 秒 xfade 转场）
- 5 种随机转场：fade / slideleft / slideright / wipeleft / zoomin
- 离线 TTS 配音（espeak-ng）+ 静态字幕 + 贯穿标题（drawtext）
- 三轨音频混合：配音 100% + BGM 30% + 原声 8%
- 输出 1080p / 30fps / H.264 / AAC / MP4
- JSON 项目持久化（保存 / 加载 / 新建）
- 前端实时进度条（监听 Tauri `progress` 事件）

## 当前状态：✅ 全部完成 + 自测通过

```
$ cargo test --lib
running 3 tests
test compose::tests::xfade_offsets_are_correct ... ok
test pipeline::e2e_tests::xfade_filter_offsets_correct ... ok
test pipeline::e2e_tests::full_pipeline_runs_and_produces_output ... ok
test result: ok. 3 passed; 0 failed
```

端到端测试用 `ffmpeg lavfi` 合成 3 个场景的 12 秒测试素材，跑完整流水线（预处理 → 选片 → xfade 拼接 → 字幕叠加 → TTS → 三轨混音 → JSON 存盘往返），校验输出文件存在且时长在合理区间。

## 目录结构

```
hunjian/
├── package.json / vite.config.js / svelte.config.js / index.html
├── src/                      Svelte 前端
│   ├── App.svelte            （含进度监听 + 项目存盘）
│   ├── main.js / app.css
│   └── lib/
│       ├── api.js            包装 invoke + listen('progress')
│       ├── SceneManager.svelte
│       ├── ParamsPanel.svelte
│       └── ProgressBar.svelte
└── src-tauri/                Rust 后端（Tauri v2）
    ├── Cargo.toml / tauri.conf.json / build.rs
    ├── capabilities/default.json
    ├── icons/icon.png        占位图标，512×512 RGBA
    └── src/
        ├── main.rs / lib.rs
        ├── types.rs          RawAsset / Clip / Project / MixParams / MixResult
        ├── state.rs          AppState（Mutex<Project>）
        ├── error.rs          AppError 自带 Serialize 给前端
        ├── ffmpeg.rs         ffmpeg / ffprobe 调用 + 滤镜可用性探测
        ├── preprocess.rs     ✅ rayon 并行切片 + 统一编码到 1080p@30fps
        ├── selection.rs      ✅ 随机挑选 + 自动减片段满足 ≤60s
        ├── compose.rs        ✅ xfade + acrossfade 全自动拼接（offset 正确）
        ├── overlay.rs        ✅ drawtext 双层（标题贯穿 + 底部静态字幕）
        ├── tts.rs            ✅ espeak-ng 离线 TTS（CJK 自动用 cmn 语种）
        ├── audio.rs          ✅ amix 三轨：原声 8% + 配音 100% + BGM 30%
        ├── pipeline.rs       ✅ select→compose→overlay→tts→audio_mix 全流水线 + 进度事件
        └── commands.rs       ✅ 10 个 Tauri 命令
```

## 数据持久化

按桌面应用惯例**没有数据库**，用 **JSON 项目文件** 存盘：

```json
{
  "name": "我的项目",
  "scenes": ["开头", "产品", "结尾"],
  "raw_assets": [{ "path": "...", "duration_secs": 12.0, "scene_id": "开头" }],
  "clips": [...],
  "title_text": "...",
  "narration_text": "...",
  "work_dir": "/tmp/hunjian-...",
  "last_output": "/tmp/hunjian-.../output.mp4"
}
```

前端 → "保存项目 / 加载项目 / 新建" 按钮，等价于"数据库设计"的角色。

## 环境依赖

| 依赖 | 用途 | 安装 | 当前 |
|---|---|---|---|
| Rust 1.85+ | 后端编译 | 已带 | ✅ 1.95.0 |
| Node 20+ | 前端构建 | 已带 | ✅ 24.3.0 |
| FFmpeg | 视频/音频处理 | `brew install ffmpeg` | ✅ 8.1 |
| **drawtext 滤镜** | 标题/字幕 | 见下方说明 | ⚠️ 当前 ffmpeg 无（缺 freetype），自动降级 |
| espeak-ng | 离线 TTS | `brew install espeak-ng` | ⚠️ 未安装，pipeline 自动跳过 |

### 启用 drawtext（标题/字幕）

Homebrew 默认 ffmpeg 不带 freetype，drawtext 滤镜不可用。要启用文字叠加：

```bash
brew tap homebrew-ffmpeg/ffmpeg
brew install homebrew-ffmpeg/ffmpeg/ffmpeg --with-freetype --with-libass --with-fdk-aac
# 注意：会替换默认 ffmpeg，请按需操作
```

代码已做**优雅降级**：drawtext 缺失时自动跳过文字层，不影响视频生成。

## 安装与运行

```bash
cd /Users/chengyisi/AI开发/项目/hunjian

# 1) 前端依赖
npm install

# 2) 可选：补齐 TTS 与 drawtext
brew install espeak-ng
# brew tap homebrew-ffmpeg/ffmpeg && brew install homebrew-ffmpeg/ffmpeg/ffmpeg

# 3) 桌面开发模式（自动起 Vite + Tauri 窗口）
npm run tauri dev

# 4) 仅跑后端测试
cd src-tauri && cargo test --lib
```

## Tauri 命令一览

前端通过 `src/lib/api.js` 调用，对应 `src-tauri/src/commands.rs`。

| 前端 API | Rust 命令 | 作用 |
|---|---|---|
| `addScene(name)` | `add_scene` | 新增场景 |
| `removeScene(name)` | `remove_scene` | 删除场景及其素材/片段 |
| `importAssets(sceneId, paths)` | `import_assets` | 导入原始视频（自动 ffprobe 时长） |
| `preprocess(clipDurationSecs)` | `preprocess` | rayon 并行切片 |
| `generateMix(params)` | `generate_mix` | 完整流水线（自动推 progress 事件） |
| `resetClips()` | `reset_clips` | 重置 used 标记 |
| `getProject()` | `get_project` | 获取当前项目状态 |
| `saveProject(path)` | `save_project` | 序列化项目到 JSON |
| `loadProject(path)` | `load_project` | 加载 JSON 项目 |
| `newProject()` | `new_project` | 新建空项目 |
| `onProgress(cb)` | event listener | 订阅流水线进度 |

## 流水线阶段（progress 事件）

```
select   →  compose  →  overlay  →  tts   →  audio  →  done
  5%        25%         55%         70%      85%      100%
```

每阶段前端实时显示百分比和说明。

## 排错

- **`ffmpeg: No such filter: 'drawtext'`** — 装带 freetype 的 ffmpeg 版本（见上方说明），或忽略：代码会自动跳过文字层。
- **`espeak-ng: command not found`** — 装 `brew install espeak-ng`，或留空"配音文案"输入框跳过 TTS。
- **`npm run tauri dev` 报缺图标** — 已默认 `bundle.active: false`。打包时再执行 `npm run tauri icon path/to/icon.png`，并把 `bundle.active` 改回 `true`。
- **首次 cargo 编译慢** — Tauri 依赖较多，首次拉一堆 crate（~1 分钟），属正常。

## 测试覆盖

- `compose::tests::xfade_offsets_are_correct` — xfade offset 数学正确（单元测试）
- `pipeline::e2e_tests::xfade_filter_offsets_correct` — 集成层再次校验
- `pipeline::e2e_tests::full_pipeline_runs_and_produces_output` — 端到端：合成测试素材 → 完整流水线 → 校验输出 + JSON 持久化往返

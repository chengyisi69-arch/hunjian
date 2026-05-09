<script>
  import { onMount, onDestroy } from "svelte";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import SceneManager from "./lib/SceneManager.svelte";
  import ParamsPanel from "./lib/ParamsPanel.svelte";
  import ProgressBar from "./lib/ProgressBar.svelte";
  import AIPanel from "./lib/AIPanel.svelte";
  import * as api from "./lib/api.js";

  let scenes = [];
  let params = {
    clipDurationSecs: 3,
    picksPerScene: 1,
    titleText: "",
    titlePosition: "5,5",
    titleFontSize: 48,
    titleStyle: "simple",
    title2Text: "",
    title2Position: "",
    title2FontSize: 48,
    title2Style: "simple",
    title3Text: "",
    title3Position: "",
    title3FontSize: 48,
    title3Style: "simple",
    subtitlePosition: "50,95",
    narrationText: "",
    bgMusicPath: null,
    maxDurationSecs: 60,
    orientation: "portrait",
    originalVolume: 8,
    narrationVolume: 100,
    bgmVolume: 30,
    ttsVoice: "auto",
    batchCount: 1,
    autoSubtitle: false,
    transSecs: 0.5,
    bgmFadeSecs: 1.5,
  };

  let progress = { stage: "idle", percent: 0, message: "" };
  let busy = false;
  let lastOutput = null;
  let batchOutputs = [];
  let unlistenProgress = null;

  onMount(async () => {
    unlistenProgress = await api.onProgress((p) => {
      progress = { stage: p.stage, percent: p.percent, message: p.message };
    });
    await refreshFromBackend();
  });

  onDestroy(() => {
    if (unlistenProgress) unlistenProgress();
  });

  async function refreshFromBackend() {
    const proj = await api.getProject();
    scenes = proj.scenes.map((name) => ({
      name,
      assets: proj.raw_assets.filter((a) => a.scene_id === name).map((a) => a.path),
    }));
    if (proj.last_output) lastOutput = proj.last_output;
    if (proj.batch_outputs) batchOutputs = proj.batch_outputs;
  }

  async function preprocess() {
    busy = true;
    progress = { stage: "preprocess", percent: 5, message: "切割素材中…" };
    try {
      const n = await api.preprocess(params.clipDurationSecs, params.orientation);
      progress = {
        stage: "preprocess-done",
        percent: 100,
        message: `预处理完成，共生成 ${n} 个片段`,
      };
    } catch (e) {
      progress = { stage: "error", percent: 0, message: String(e) };
    } finally {
      busy = false;
    }
  }

  async function generate() {
    busy = true;
    progress = { stage: "generate", percent: 0, message: "生成中…" };
    try {
      const payload = {
        ...params,
        originalVolume: params.originalVolume / 100,
        narrationVolume: params.narrationVolume / 100,
        bgmVolume: params.bgmVolume / 100,
      };
      const result = await api.generateMix(payload);
      batchOutputs = result.outputs.map((r) => r.output_path);
      if (batchOutputs.length > 0) lastOutput = batchOutputs[0];
      progress = {
        stage: "done",
        percent: 100,
        message: `已生成 ${result.total_count} 个视频`,
      };
    } catch (e) {
      progress = { stage: "error", percent: 0, message: String(e) };
    } finally {
      busy = false;
    }
  }

  async function reset() {
    await api.resetClips();
    progress = { stage: "idle", percent: 0, message: "已重置片段使用状态" };
  }

  async function newProject() {
    if (busy) return;
    await api.newProject();
    await refreshFromBackend();
    lastOutput = null;
    batchOutputs = [];
    progress = { stage: "idle", percent: 0, message: "已新建项目" };
  }

  async function saveProject() {
    const path = await save({
      defaultPath: "project.hunjian.json",
      filters: [{ name: "Hunjian Project", extensions: ["json"] }],
    });
    if (!path) return;
    await api.saveProject(path);
    progress = { stage: "idle", percent: 0, message: `已保存: ${path}` };
  }

  async function loadProject() {
    const path = await open({
      multiple: false,
      filters: [{ name: "Hunjian Project", extensions: ["json"] }],
    });
    if (!path) return;
    await api.loadProject(path);
    await refreshFromBackend();
    progress = { stage: "idle", percent: 0, message: `已加载: ${path}` };
  }

  async function downloadVideo(path) {
    if (!path) return;
    const dst = await save({
      defaultPath: path.split("/").pop() || "output.mp4",
      filters: [{ name: "MP4 Video", extensions: ["mp4"] }],
    });
    if (!dst) return;
    try {
      await api.exportVideo(path, dst);
      progress = { stage: "idle", percent: 0, message: `已下载到: ${dst}` };
    } catch (e) {
      progress = { stage: "error", percent: 0, message: String(e) };
    }
  }

  async function batchDownload() {
    if (!batchOutputs.length) return;
    const dir = await open({
      directory: true,
      multiple: false,
    });
    if (!dir) return;
    busy = true;
    progress = { stage: "batch-download", percent: 5, message: "批量下载中…" };
    try {
      await Promise.all(
        batchOutputs.map((src) => {
          const name = src.split("/").pop() || "output.mp4";
          const dst = `${dir}/${name}`;
          return api.exportVideo(src, dst);
        })
      );
      progress = {
        stage: "idle",
        percent: 100,
        message: `已批量下载 ${batchOutputs.length} 个视频到 ${dir}`,
      };
    } catch (e) {
      progress = { stage: "error", percent: 0, message: String(e) };
    } finally {
      busy = false;
    }
  }
</script>

<main>
  <header>
    <h1>混剪生成器</h1>
    <div class="topbar">
      <button on:click={newProject} disabled={busy}>新建</button>
      <button on:click={saveProject} disabled={busy}>保存项目</button>
      <button on:click={loadProject} disabled={busy}>加载项目</button>
    </div>
  </header>

  <section>
    <AIPanel bind:params {scenes} />
  </section>

  <section>
    <SceneManager bind:scenes />
  </section>

  <section>
    <ParamsPanel bind:params />
  </section>

  <section class="actions">
    <button on:click={preprocess} disabled={busy || scenes.length === 0}>预处理素材</button>
    <button on:click={generate} disabled={busy || scenes.length === 0}>生成混剪</button>
    <button on:click={reset} disabled={busy}>重置片段状态</button>
  </section>

  <section>
    <ProgressBar {progress} />
    {#if batchOutputs.length > 0}
      <div class="output">
        <div class="output-header">
          生成结果（{batchOutputs.length} 个）：
          <button class="batch-download" on:click={batchDownload} disabled={busy}>批量下载全部</button>
        </div>
        {#each batchOutputs as path, i}
          <div class="output-item">
            <code>{path}</code>
            <button on:click={() => downloadVideo(path)} disabled={busy}>下载</button>
          </div>
        {/each}
      </div>
    {/if}
  </section>
</main>

<style>
  main {
    max-width: 980px;
    margin: 0 auto;
    padding: 24px;
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }
  header h1 {
    margin: 0;
    font-size: 22px;
  }
  .topbar {
    display: flex;
    gap: 8px;
  }
  .topbar button {
    padding: 6px 12px;
    border: 1px solid #d0d4d9;
    background: #fff;
    border-radius: 6px;
  }
  section {
    background: #fff;
    border: 1px solid #e6e8eb;
    border-radius: 8px;
    padding: 16px;
    margin-bottom: 16px;
  }
  .actions {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
  }
  .actions button {
    padding: 8px 16px;
    border: 1px solid #d0d4d9;
    background: #f3f5f8;
    border-radius: 6px;
  }
  .output {
    margin-top: 12px;
    font-size: 13px;
    color: #333;
  }
  .output-header {
    font-weight: 600;
    margin-bottom: 8px;
  }
  .output-item {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
    word-break: break-all;
  }
  .output-item code {
    background: #f3f5f8;
    padding: 2px 6px;
    border-radius: 3px;
    flex: 1;
  }
  .output-item button {
    padding: 4px 10px;
    border: 1px solid #d0d4d9;
    background: #f3f5f8;
    border-radius: 4px;
    white-space: nowrap;
  }
  .batch-download {
    padding: 4px 12px;
    border: 1px solid #4a7cff;
    background: #4a7cff;
    color: #fff;
    border-radius: 4px;
    margin-left: 12px;
    font-size: 13px;
  }
</style>

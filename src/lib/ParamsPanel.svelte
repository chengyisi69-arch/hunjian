<script>
  import { open } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import PositionPicker from "./PositionPicker.svelte";
  import * as api from "./api.js";

  export let params;

  let templates = [];
  let newTemplateName = "";

  onMount(async () => {
    await refreshTemplates();
  });

  async function refreshTemplates() {
    try {
      templates = await api.listTemplates();
    } catch (e) {
      console.error("加载模板失败", e);
    }
  }

  async function saveTemplate() {
    if (!newTemplateName.trim()) return;
    const template = {
      name: newTemplateName.trim(),
      clip_duration_secs: params.clipDurationSecs,
      picks_per_scene: params.picksPerScene,
      max_duration_secs: params.maxDurationSecs,
      orientation: params.orientation,
      title_font_size: params.titleFontSize,
      title_style: params.titleStyle,
      title2_font_size: params.title2FontSize,
      title2_style: params.title2Style,
      title3_font_size: params.title3FontSize,
      title3_style: params.title3Style,
      original_volume: params.originalVolume / 100,
      narration_volume: params.narrationVolume / 100,
      bgm_volume: params.bgmVolume / 100,
      tts_voice: params.ttsVoice,
      batch_count: params.batchCount,
      auto_subtitle: params.autoSubtitle,
      trans_secs: params.transSecs,
      bgm_fade_secs: params.bgmFadeSecs,
    };
    await api.saveTemplate(template);
    newTemplateName = "";
    await refreshTemplates();
  }

  async function applyTemplateByName(name) {
    const t = await api.applyTemplate(name);
    params.clipDurationSecs = t.clip_duration_secs;
    params.picksPerScene = t.picks_per_scene;
    params.maxDurationSecs = t.max_duration_secs;
    params.orientation = t.orientation;
    params.titleFontSize = t.title_font_size;
    params.titleStyle = t.title_style;
    params.title2FontSize = t.title2_font_size;
    params.title2Style = t.title2_style;
    params.title3FontSize = t.title3_font_size;
    params.title3Style = t.title3_style;
    params.originalVolume = Math.round(t.original_volume * 100);
    params.narrationVolume = Math.round(t.narration_volume * 100);
    params.bgmVolume = Math.round(t.bgm_volume * 100);
    params.ttsVoice = t.tts_voice;
    params.batchCount = t.batch_count;
    params.autoSubtitle = t.auto_subtitle;
    params.transSecs = t.trans_secs;
    params.bgmFadeSecs = t.bgm_fade_secs;
  }

  async function deleteTemplateByName(name) {
    await api.deleteTemplate(name);
    await refreshTemplates();
  }

  async function pickBgMusic() {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Audio", extensions: ["mp3", "wav", "m4a", "aac", "flac"] }],
    });
    if (selected) params.bgMusicPath = selected;
  }

  const styleOptions = [
    { value: "simple", label: "简约白字" },
    { value: "outline", label: "描边黑字" },
    { value: "shadow", label: "投影阴影" },
    { value: "neon", label: "霓虹描边" },
    { value: "gradient", label: "渐变蓝紫" },
    { value: "vintage", label: "复古米黄" },
    { value: "tech", label: "科技青色" },
    { value: "comic", label: "漫画粗边" },
    { value: "golden", label: "金色描边" },
    { value: "clean", label: "极简白字" },
  ];

  function titlePreviewText(idx) {
    if (idx === 1) return params.titleText || "标题1预览";
    if (idx === 2) return params.title2Text || "标题2预览";
    return params.title3Text || "标题3预览";
  }
</script>

<h2>参数设置</h2>

<!-- 模板管理 -->
<div class="template-bar">
  <div class="template-input">
    <input type="text" bind:value={newTemplateName} placeholder="新模板名称" />
    <button on:click={saveTemplate}>保存模板</button>
  </div>
  {#if templates.length > 0}
    <div class="template-list">
      {#each templates as t}
        <div class="template-chip">
          <span on:click={() => applyTemplateByName(t.name)}>{t.name}</span>
          <button on:click={() => deleteTemplateByName(t.name)}>×</button>
        </div>
      {/each}
    </div>
  {/if}
</div>

<div class="grid">
  <label>
    片段长度（秒）
    <select bind:value={params.clipDurationSecs}>
      {#each Array.from({ length: 10 }, (_, i) => i + 1) as n}
        <option value={n}>{n}</option>
      {/each}
    </select>
  </label>

  <label>
    每个场景片段数量
    <select bind:value={params.picksPerScene}>
      {#each Array.from({ length: 10 }, (_, i) => i + 1) as n}
        <option value={n}>{n}</option>
      {/each}
    </select>
  </label>

  <label>
    最大时长（秒）
    <input type="number" min="10" max="100" bind:value={params.maxDurationSecs} />
  </label>

  <label>
    批量生成数量
    <input type="number" min="1" max="50" bind:value={params.batchCount} />
  </label>

  <label>
    转场时长（秒）
    <select bind:value={params.transSecs}>
      {#each [0, 0.3, 0.5, 0.7, 1.0, 1.2, 1.5] as t}
        <option value={t}>{t}</option>
      {/each}
    </select>
  </label>

  <label>
    方向
    <select bind:value={params.orientation}>
      <option value="portrait">竖屏（1080×1920）</option>
      <option value="landscape">横屏（1920×1080）</option>
    </select>
  </label>

  <label>
    配音音色
    <select bind:value={params.ttsVoice}>
      <option value="auto">自动</option>
      <option value="female">标准女声</option>
      <option value="male">标准男声</option>
      <option value="news_female">女播音</option>
      <option value="loli">萝莉</option>
      <option value="uncle">大叔</option>
      <option value="youth_male">青年男声</option>
      <option value="sweet_female">甜美女声</option>
      <option value="deep_male">磁性低音</option>
    </select>
  </label>

  <label class="full checkbox-row">
    <input type="checkbox" bind:checked={params.autoSubtitle} />
    <span>启用自动语音字幕（按句子显示时间轴）</span>
  </label>

  <label class="full">
    配音文案（TTS）
    <textarea rows="4" bind:value={params.narrationText} placeholder="将转为语音作为旁白，同时作为字幕来源" />
  </label>

  <label class="full">
    背景音乐（可选）
    <div class="row">
      <input type="text" readonly value={params.bgMusicPath ?? ""} placeholder="未选择" />
      <button on:click={pickBgMusic}>选择文件</button>
      {#if params.bgMusicPath}
        <button on:click={() => (params.bgMusicPath = null)}>清除</button>
      {/if}
    </div>
  </label>

  <label>
    BGM 淡入淡出（秒）
    <select bind:value={params.bgmFadeSecs}>
      {#each [0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0] as t}
        <option value={t}>{t}</option>
      {/each}
    </select>
  </label>

  <label class="full">
    音量调节（0–200%）
    <div class="volume-row">
      <div>
        <span>原片段声</span>
        <input type="range" min="0" max="200" step="5" bind:value={params.originalVolume} />
        <span>{params.originalVolume}%</span>
      </div>
      <div>
        <span>配音</span>
        <input type="range" min="0" max="200" step="5" bind:value={params.narrationVolume} />
        <span>{params.narrationVolume}%</span>
      </div>
      <div>
        <span>背景音乐</span>
        <input type="range" min="0" max="200" step="5" bind:value={params.bgmVolume} />
        <span>{params.bgmVolume}%</span>
      </div>
    </div>
  </label>

  <!-- 标题1 -->
  <div class="full title-section">
    <div class="title-header">标题 1</div>
    <div class="title-grid">
      <label class="full">
        文字
        <input type="text" bind:value={params.titleText} placeholder="输入标题文字" />
      </label>
      <label>
        字号
        <select bind:value={params.titleFontSize}>
          {#each [24, 32, 40, 48, 56, 64, 72, 96] as sz}
            <option value={sz}>{sz}</option>
          {/each}
        </select>
      </label>
      <label>
        样式
        <select bind:value={params.titleStyle}>
          {#each styleOptions as opt}
            <option value={opt.value}>{opt.label}</option>
          {/each}
        </select>
      </label>
      <div class="picker-wrap">
        <PositionPicker
          label="位置"
          bind:value={params.titlePosition}
          orientation={params.orientation}
          accentColor="#4a7cff"
          text={titlePreviewText(1)}
          fontSize={params.titleFontSize}
          style={params.titleStyle}
        />
      </div>
    </div>
  </div>

  <!-- 标题2 -->
  <div class="full title-section">
    <div class="title-header">标题 2（可选）</div>
    <div class="title-grid">
      <label class="full">
        文字
        <input type="text" bind:value={params.title2Text} placeholder="输入标题文字（留空则不显示）" />
      </label>
      <label>
        字号
        <select bind:value={params.title2FontSize}>
          {#each [24, 32, 40, 48, 56, 64, 72, 96] as sz}
            <option value={sz}>{sz}</option>
          {/each}
        </select>
      </label>
      <label>
        样式
        <select bind:value={params.title2Style}>
          {#each styleOptions as opt}
            <option value={opt.value}>{opt.label}</option>
          {/each}
        </select>
      </label>
      <div class="picker-wrap">
        <PositionPicker
          label="位置"
          bind:value={params.title2Position}
          orientation={params.orientation}
          accentColor="#ff6b6b"
          text={titlePreviewText(2)}
          fontSize={params.title2FontSize}
          style={params.title2Style}
        />
      </div>
    </div>
  </div>

  <!-- 标题3 -->
  <div class="full title-section">
    <div class="title-header">标题 3（可选）</div>
    <div class="title-grid">
      <label class="full">
        文字
        <input type="text" bind:value={params.title3Text} placeholder="输入标题文字（留空则不显示）" />
      </label>
      <label>
        字号
        <select bind:value={params.title3FontSize}>
          {#each [24, 32, 40, 48, 56, 64, 72, 96] as sz}
            <option value={sz}>{sz}</option>
          {/each}
        </select>
      </label>
      <label>
        样式
        <select bind:value={params.title3Style}>
          {#each styleOptions as opt}
            <option value={opt.value}>{opt.label}</option>
          {/each}
        </select>
      </label>
      <div class="picker-wrap">
        <PositionPicker
          label="位置"
          bind:value={params.title3Position}
          orientation={params.orientation}
          accentColor="#22c55e"
          text={titlePreviewText(3)}
          fontSize={params.title3FontSize}
          style={params.title3Style}
        />
      </div>
    </div>
  </div>

  <!-- 字幕位置 -->
  <div class="full title-section">
    <div class="title-header">字幕位置</div>
    <div class="picker-wrap">
      <PositionPicker
        label="字幕位置"
        bind:value={params.subtitlePosition}
        orientation={params.orientation}
        accentColor="#f59e0b"
        text={params.narrationText ? params.narrationText.slice(0, 12) + "..." : "字幕预览"}
        fontSize={32}
        style="simple"
      />
    </div>
  </div>
</div>

<style>
  h2 {
    margin: 0 0 12px;
    font-size: 16px;
  }
  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }
  .grid .full {
    grid-column: 1 / -1;
  }
  label {
    display: flex;
    flex-direction: column;
    font-size: 13px;
    color: #555;
    gap: 4px;
  }
  input,
  select,
  textarea {
    padding: 6px 8px;
    border: 1px solid #d0d4d9;
    border-radius: 4px;
    font-size: 14px;
    color: #222;
  }
  .row {
    display: flex;
    gap: 8px;
  }
  .row input {
    flex: 1;
  }
  .volume-row {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .volume-row > div {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .volume-row span:first-child {
    width: 72px;
    text-align: right;
    font-size: 13px;
    color: #444;
  }
  .volume-row span:last-child {
    width: 44px;
    text-align: left;
    font-size: 13px;
    color: #444;
  }
  .volume-row input[type="range"] {
    flex: 1;
  }
  .checkbox-row {
    flex-direction: row;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    color: #333;
  }
  .checkbox-row input {
    width: 18px;
    height: 18px;
  }
  .title-section {
    background: #f8f9fb;
    border: 1px solid #e6e8eb;
    border-radius: 6px;
    padding: 12px;
  }
  .title-header {
    font-size: 14px;
    font-weight: 600;
    color: #333;
    margin-bottom: 10px;
  }
  .title-grid {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    gap: 10px;
  }
  .title-grid .full {
    grid-column: 1 / -1;
  }
  .picker-wrap {
    display: flex;
    justify-content: center;
    grid-column: 1 / -1;
  }
  button {
    padding: 6px 10px;
    border: 1px solid #d0d4d9;
    background: #f3f5f8;
    border-radius: 4px;
  }
  .template-bar {
    margin-bottom: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .template-input {
    display: flex;
    gap: 8px;
  }
  .template-input input {
    flex: 1;
  }
  .template-list {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .template-chip {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    background: #eef2ff;
    border: 1px solid #c7d2fe;
    border-radius: 16px;
    font-size: 13px;
    color: #374151;
  }
  .template-chip span {
    cursor: pointer;
  }
  .template-chip span:hover {
    color: #4a7cff;
  }
  .template-chip button {
    padding: 0 4px;
    border: none;
    background: transparent;
    color: #9ca3af;
    font-size: 14px;
    line-height: 1;
  }
  .template-chip button:hover {
    color: #ef4444;
  }
</style>

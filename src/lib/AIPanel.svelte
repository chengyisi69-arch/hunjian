<script>
  import { onMount } from "svelte";
  import * as api from "./api.js";

  export let params;
  export let scenes = [];

  let aiPrompt = "";
  let generatedTitles = [];
  let generatedScript = "";
  let generatedImagePath = "";
  let aiBusy = false;
  let aiMessage = "";

  // AI 配置
  let showConfig = false;
  let aiConfig = {
    llm_api_key: "",
    llm_api_url: "https://api.openai.com/v1/chat/completions",
    llm_model: "gpt-4o-mini",
    image_api_key: "",
    image_api_url: "https://api.openai.com/v1/images/generations",
    image_model: "dall-e-3",
    video_api_key: "",
    video_api_url: "",
    video_model: "",
  };

  // 图片转视频
  let motionEffect = "zoom-in";
  let targetScene = "";
  let imageToVideoPath = "";
  let videoBusy = false;

  onMount(async () => {
    try {
      aiConfig = await api.getAiConfig();
    } catch (e) {
      console.error("加载 AI 配置失败", e);
    }
  });

  async function saveAiConfig() {
    try {
      await api.setAiConfig(aiConfig);
      aiMessage = "配置已保存";
      setTimeout(() => (aiMessage = ""), 2000);
    } catch (e) {
      aiMessage = "保存失败: " + String(e);
    }
  }

  async function genTitle() {
    if (!aiPrompt.trim()) return;
    aiBusy = true;
    aiMessage = "正在生成标题…";
    try {
      const text = await api.aiGenerateTitle(aiPrompt);
      generatedTitles = text
        .split("\n")
        .map((t) => t.trim())
        .filter((t) => t.length > 0 && !t.match(/^\d+\./));
      aiMessage = "标题生成完成";
    } catch (e) {
      aiMessage = "生成失败: " + String(e);
    } finally {
      aiBusy = false;
    }
  }

  async function genScript() {
    if (!aiPrompt.trim()) return;
    aiBusy = true;
    aiMessage = "正在生成文案…";
    try {
      generatedScript = await api.aiGenerateScript(aiPrompt);
      aiMessage = "文案生成完成";
    } catch (e) {
      aiMessage = "生成失败: " + String(e);
    } finally {
      aiBusy = false;
    }
  }

  async function genImage() {
    if (!aiPrompt.trim()) return;
    aiBusy = true;
    aiMessage = "正在生成配图…";
    try {
      generatedImagePath = await api.aiGenerateImage(aiPrompt);
      imageToVideoPath = generatedImagePath;
      aiMessage = "配图生成完成";
    } catch (e) {
      aiMessage = "生成失败: " + String(e);
    } finally {
      aiBusy = false;
    }
  }

  function useTitle(t) {
    params.titleText = t;
  }

  function useScript() {
    params.narrationText = generatedScript;
  }

  async function convertToVideo() {
    if (!imageToVideoPath) return;
    videoBusy = true;
    aiMessage = "正在转换动态片段…";
    try {
      const duration = params.clipDurationSecs || 3;
      const out = await api.imageToVideo(
        imageToVideoPath,
        duration,
        motionEffect,
        params.orientation
      );
      aiMessage = "动态片段生成完成";
      // 如果有目标场景，自动添加为素材
      if (targetScene) {
        await api.addImageAsAsset(
          targetScene,
          imageToVideoPath,
          duration,
          params.orientation
        );
        aiMessage = "已添加到场景: " + targetScene;
      }
      imageToVideoPath = out;
    } catch (e) {
      aiMessage = "转换失败: " + String(e);
    } finally {
      videoBusy = false;
    }
  }

  async function addImageToScene() {
    if (!generatedImagePath || !targetScene) {
      aiMessage = "请先生成图片并选择目标场景";
      return;
    }
    aiBusy = true;
    aiMessage = "正在添加素材…";
    try {
      const duration = params.clipDurationSecs || 3;
      await api.addImageAsAsset(
        targetScene,
        generatedImagePath,
        duration,
        params.orientation
      );
      aiMessage = "已添加到场景: " + targetScene;
    } catch (e) {
      aiMessage = "添加失败: " + String(e);
    } finally {
      aiBusy = false;
    }
  }
</script>

<div class="ai-panel">
  <div class="ai-header">
    <h3>AI 智能创作</h3>
    <button class="config-btn" on:click={() => (showConfig = !showConfig)}>
      {showConfig ? "收起配置" : "API 配置"}
    </button>
  </div>

  {#if showConfig}
    <div class="config-section">
      <div class="config-grid">
        <label>
          LLM API Key
          <input type="password" bind:value={aiConfig.llm_api_key} placeholder="sk-..." />
        </label>
        <label>
          LLM 模型
          <input type="text" bind:value={aiConfig.llm_model} />
        </label>
        <label class="full">
          LLM API URL
          <input type="text" bind:value={aiConfig.llm_api_url} />
        </label>
        <label>
          图片 API Key
          <input type="password" bind:value={aiConfig.image_api_key} placeholder="sk-..." />
        </label>
        <label>
          图片模型
          <input type="text" bind:value={aiConfig.image_model} />
        </label>
        <label class="full">
          图片 API URL
          <input type="text" bind:value={aiConfig.image_api_url} />
        </label>
      </div>
      <button class="save-btn" on:click={saveAiConfig}>保存配置</button>
    </div>
  {/if}

  <div class="ai-input">
    <label class="full">
      视频主题 / 描述
      <textarea
        rows="2"
        bind:value={aiPrompt}
        placeholder="例如：夏日海滩旅行 Vlog，阳光沙滩冲浪"
      />
    </label>

    <div class="ai-actions">
      <button on:click={genTitle} disabled={aiBusy || !aiPrompt.trim()}>生成标题</button>
      <button on:click={genScript} disabled={aiBusy || !aiPrompt.trim()}>生成文案</button>
      <button on:click={genImage} disabled={aiBusy || !aiPrompt.trim()}>生成配图</button>
    </div>

    {#if aiMessage}
      <div class="ai-message">{aiMessage}</div>
    {/if}
  </div>

  {#if generatedTitles.length > 0}
    <div class="result-section">
      <div class="result-header">候选标题：</div>
      <div class="title-list">
        {#each generatedTitles as t}
          <div class="title-chip">
            <span>{t}</span>
            <button on:click={() => useTitle(t)}>使用</button>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  {#if generatedScript}
    <div class="result-section">
      <div class="result-header">生成文案：</div>
      <textarea rows="3" readonly value={generatedScript} />
      <button on:click={useScript}>填入配音文案</button>
    </div>
  {/if}

  {#if generatedImagePath}
    <div class="result-section">
      <div class="result-header">生成配图：</div>
      <img src="src-file://{generatedImagePath}" alt="AI 配图" class="ai-image" />

      <div class="motion-controls">
        <label>
          动态效果
          <select bind:value={motionEffect}>
            <option value="zoom-in">缓慢缩放</option>
            <option value="ken-burns">Ken Burns</option>
            <option value="pan-left">向左平移</option>
            <option value="pan-right">向右平移</option>
          </select>
        </label>
        <label>
          目标场景
          <select bind:value={targetScene}>
            <option value="">选择场景…</option>
            {#each scenes as s}
              <option value={s.name}>{s.name}</option>
            {/each}
          </select>
        </label>
        <div class="motion-actions">
          <button on:click={convertToVideo} disabled={videoBusy}>转为动态片段</button>
          <button on:click={addImageToScene} disabled={aiBusy}>直接添加为素材</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .ai-panel {
    background: #f0f4ff;
    border: 1px solid #c7d2fe;
    border-radius: 8px;
    padding: 16px;
    margin-bottom: 16px;
  }
  .ai-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }
  .ai-header h3 {
    margin: 0;
    font-size: 15px;
    color: #1e3a5f;
  }
  .config-btn {
    padding: 4px 10px;
    border: 1px solid #a5b4fc;
    background: #e0e7ff;
    border-radius: 4px;
    font-size: 12px;
    color: #4338ca;
  }
  .config-section {
    background: #fff;
    border-radius: 6px;
    padding: 12px;
    margin-bottom: 12px;
  }
  .config-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
    margin-bottom: 10px;
  }
  .config-grid .full {
    grid-column: 1 / -1;
  }
  .config-grid label {
    display: flex;
    flex-direction: column;
    font-size: 12px;
    color: #555;
    gap: 4px;
  }
  .config-grid input {
    padding: 5px 8px;
    border: 1px solid #d0d4d9;
    border-radius: 4px;
    font-size: 13px;
  }
  .save-btn {
    padding: 5px 14px;
    border: 1px solid #4a7cff;
    background: #4a7cff;
    color: #fff;
    border-radius: 4px;
    font-size: 13px;
  }
  .ai-input label {
    display: flex;
    flex-direction: column;
    font-size: 13px;
    color: #444;
    gap: 4px;
    margin-bottom: 10px;
  }
  .ai-input textarea {
    padding: 8px;
    border: 1px solid #d0d4d9;
    border-radius: 4px;
    font-size: 14px;
    resize: vertical;
  }
  .ai-actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    margin-bottom: 8px;
  }
  .ai-actions button {
    padding: 6px 14px;
    border: 1px solid #4a7cff;
    background: #4a7cff;
    color: #fff;
    border-radius: 4px;
    font-size: 13px;
  }
  .ai-actions button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .ai-message {
    font-size: 12px;
    color: #4a7cff;
    margin-bottom: 8px;
  }
  .result-section {
    background: #fff;
    border-radius: 6px;
    padding: 12px;
    margin-top: 10px;
  }
  .result-header {
    font-size: 13px;
    font-weight: 600;
    color: #333;
    margin-bottom: 8px;
  }
  .title-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .title-chip {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 10px;
    background: #f8f9fb;
    border: 1px solid #e6e8eb;
    border-radius: 4px;
  }
  .title-chip span {
    font-size: 14px;
    color: #222;
  }
  .title-chip button {
    padding: 3px 10px;
    border: 1px solid #4a7cff;
    background: #fff;
    color: #4a7cff;
    border-radius: 4px;
    font-size: 12px;
  }
  .result-section textarea {
    width: 100%;
    padding: 8px;
    border: 1px solid #d0d4d9;
    border-radius: 4px;
    font-size: 13px;
    margin-bottom: 8px;
    box-sizing: border-box;
  }
  .result-section button {
    padding: 5px 12px;
    border: 1px solid #4a7cff;
    background: #fff;
    color: #4a7cff;
    border-radius: 4px;
    font-size: 12px;
  }
  .ai-image {
    max-width: 100%;
    max-height: 240px;
    border-radius: 6px;
    margin-bottom: 10px;
    display: block;
  }
  .motion-controls {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }
  .motion-controls label {
    display: flex;
    flex-direction: column;
    font-size: 12px;
    color: #555;
    gap: 4px;
  }
  .motion-controls select {
    padding: 5px 8px;
    border: 1px solid #d0d4d9;
    border-radius: 4px;
    font-size: 13px;
  }
  .motion-actions {
    grid-column: 1 / -1;
    display: flex;
    gap: 8px;
  }
  .motion-actions button {
    padding: 5px 12px;
    border: 1px solid #4a7cff;
    background: #4a7cff;
    color: #fff;
    border-radius: 4px;
    font-size: 12px;
  }
  .motion-actions button:disabled {
    opacity: 0.5;
  }
</style>

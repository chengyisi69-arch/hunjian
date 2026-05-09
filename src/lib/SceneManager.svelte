<script>
  import { open } from "@tauri-apps/plugin-dialog";
  import * as api from "./api.js";

  export let scenes = [];
  let newSceneName = "";

  async function addScene() {
    const name = newSceneName.trim();
    if (!name || scenes.find((s) => s.name === name)) return;
    await api.addScene(name);
    scenes = [...scenes, { name, assets: [] }];
    newSceneName = "";
  }

  async function removeScene(name) {
    await api.removeScene(name);
    scenes = scenes.filter((s) => s.name !== name);
  }

  async function pickAssets(scene) {
    const selected = await open({
      multiple: true,
      filters: [{ name: "Video", extensions: ["mp4", "mov", "mkv", "avi", "webm"] }],
    });
    if (!selected) return;
    const paths = Array.isArray(selected) ? selected : [selected];
    await api.importAssets(scene.name, paths);
    scene.assets = [...scene.assets, ...paths];
    scenes = scenes;
  }
</script>

<h2>场景管理</h2>

<div class="add-row">
  <input
    type="text"
    placeholder="新场景名称（如：开头、产品、结尾）"
    bind:value={newSceneName}
    on:keydown={(e) => e.key === "Enter" && addScene()}
  />
  <button on:click={addScene}>添加场景</button>
</div>

{#if scenes.length === 0}
  <p class="hint">请先添加至少一个场景。</p>
{/if}

<ul>
  {#each scenes as scene (scene.name)}
    <li>
      <div class="scene-head">
        <strong>{scene.name}</strong>
        <span class="count">{scene.assets.length} 个素材</span>
        <button on:click={() => pickAssets(scene)}>导入素材</button>
        <button on:click={() => removeScene(scene.name)}>删除</button>
      </div>
      {#if scene.assets.length}
        <ul class="assets">
          {#each scene.assets as p}
            <li title={p}>{p.split("/").pop()}</li>
          {/each}
        </ul>
      {/if}
    </li>
  {/each}
</ul>

<style>
  h2 {
    margin: 0 0 12px;
    font-size: 16px;
  }
  .add-row {
    display: flex;
    gap: 8px;
    margin-bottom: 12px;
  }
  .add-row input {
    flex: 1;
    padding: 6px 8px;
    border: 1px solid #d0d4d9;
    border-radius: 4px;
  }
  .hint {
    color: #888;
    font-size: 13px;
  }
  ul {
    list-style: none;
    padding-left: 0;
    margin: 0;
  }
  li {
    padding: 8px 0;
    border-bottom: 1px solid #f0f1f3;
  }
  li:last-child {
    border-bottom: none;
  }
  .scene-head {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .count {
    color: #888;
    font-size: 13px;
  }
  .assets {
    margin-top: 6px;
    padding-left: 16px;
    font-size: 13px;
    color: #555;
  }
  .assets li {
    border: none;
    padding: 2px 0;
  }
  button {
    padding: 4px 10px;
    border: 1px solid #d0d4d9;
    background: #f3f5f8;
    border-radius: 4px;
  }
</style>

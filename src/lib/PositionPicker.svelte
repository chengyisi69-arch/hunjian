<script>
  export let label = "位置";
  export let value = "50,50"; // "x,y" 百分比
  export let orientation = "portrait";
  export let accentColor = "#4a7cff";
  export let text = ""; // 预览文字
  export let fontSize = 48; // 实际视频中的字号（px）
  export let style = "simple"; // simple / outline / shadow / neon

  let containerEl;

  const PREVIEW_W = 160;
  $: previewH = orientation === "landscape" ? PREVIEW_W * 9 / 16 : PREVIEW_W * 16 / 9;

  $: coords = parseCoords(value);

  // 预览框里的字号：按视频 1080px 宽度等比缩小，再乘个系数保证可读
  $: previewFontSize = Math.max(8, Math.round(fontSize * (PREVIEW_W / 1080) * 1.6));

  function parseCoords(v) {
    if (!v || !v.includes(",")) return { x: 50, y: 50 };
    const [x, y] = v.split(",").map((n) => parseInt(n.trim(), 10));
    return { x: isNaN(x) ? 50 : x, y: isNaN(y) ? 50 : y };
  }

  function handleClick(e) {
    if (!containerEl) return;
    const rect = containerEl.getBoundingClientRect();
    const x = ((e.clientX - rect.left) / rect.width) * 100;
    const y = ((e.clientY - rect.top) / rect.height) * 100;
    const clampedX = Math.max(0, Math.min(100, Math.round(x)));
    const clampedY = Math.max(0, Math.min(100, Math.round(y)));
    value = `${clampedX},${clampedY}`;
  }

  function styleClass(s) {
    return `style-${s}`;
  }
</script>

<div class="picker-wrapper">
  <div class="picker-label">{label}：{value}</div>
  <div
    class="picker-box"
    bind:this={containerEl}
    style="width: {PREVIEW_W}px; height: {previewH}px;"
    on:click={handleClick}
  >
    <div class="grid-lines">
      <div class="h-line" style="top: 33.33%;"></div>
      <div class="h-line" style="top: 66.66%;"></div>
      <div class="v-line" style="left: 33.33%;"></div>
      <div class="v-line" style="left: 66.66%;"></div>
    </div>

    {#if text}
      <div
        class="preview-text {styleClass(style)}"
        style="left: {coords.x}%; top: {coords.y}%; font-size: {previewFontSize}px;"
      >
        {text}
      </div>
    {:else}
      <div
        class="marker"
        style="left: {coords.x}%; top: {coords.y}%; background: {accentColor};"
      ></div>
    {/if}
  </div>
  <div class="preset-buttons">
    {#each [
      { label: "左上", v: "5,5" },
      { label: "中上", v: "50,5" },
      { label: "右上", v: "95,5" },
      { label: "左中", v: "5,50" },
      { label: "正中", v: "50,50" },
      { label: "右中", v: "95,50" },
      { label: "左下", v: "5,95" },
      { label: "中下", v: "50,95" },
      { label: "右下", v: "95,95" },
    ] as preset}
      <button on:click={() => (value = preset.v)}>{preset.label}</button>
    {/each}
  </div>
</div>

<style>
  .picker-wrapper {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    margin: 8px 0;
  }
  .picker-label {
    font-size: 12px;
    color: #666;
  }
  .picker-box {
    position: relative;
    background: #1a1a2e;
    border: 2px solid #333;
    border-radius: 4px;
    cursor: crosshair;
    overflow: hidden;
  }
  .grid-lines {
    position: absolute;
    inset: 0;
    pointer-events: none;
  }
  .h-line {
    position: absolute;
    left: 0;
    right: 0;
    height: 1px;
    background: rgba(255, 255, 255, 0.15);
  }
  .v-line {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 1px;
    background: rgba(255, 255, 255, 0.15);
  }
  .marker {
    position: absolute;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    transform: translate(-50%, -50%);
    box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.8);
    pointer-events: none;
  }

  /* 预览文字基础 */
  .preview-text {
    position: absolute;
    transform: translate(-50%, -50%);
    white-space: nowrap;
    pointer-events: none;
    line-height: 1.2;
    font-family: -apple-system, BlinkMacSystemFont, "PingFang SC", "Microsoft YaHei", sans-serif;
    font-weight: 600;
    max-width: 90%;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* simple：白字 + 半透明黑底 */
  .style-simple {
    color: #fff;
    background: rgba(0, 0, 0, 0.5);
    padding: 2px 6px;
    border-radius: 3px;
  }

  /* outline：白字 + 黑描边 */
  .style-outline {
    color: #fff;
    text-shadow:
      -1px -1px 0 #000,
       1px -1px 0 #000,
      -1px  1px 0 #000,
       1px  1px 0 #000;
  }

  /* shadow：白字 + 投影 */
  .style-shadow {
    color: #fff;
    text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.8);
  }

  /* neon：黄字 + 红描边光晕 */
  .style-neon {
    color: #ffe135;
    text-shadow:
      -1px -1px 0 #ff2222,
       1px -1px 0 #ff2222,
      -1px  1px 0 #ff2222,
       1px  1px 0 #ff2222,
       0   0   4px #ff2222,
       0   0   8px #ff2222;
  }

  .preset-buttons {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 4px;
    width: 160px;
  }
  .preset-buttons button {
    padding: 3px 0;
    font-size: 11px;
    border: 1px solid #d0d4d9;
    background: #f3f5f8;
    border-radius: 3px;
    cursor: pointer;
  }
  .preset-buttons button:hover {
    background: #e8eaf0;
  }
</style>

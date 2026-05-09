<script>
  export let progress = { stage: "idle", percent: 0, message: "" };
</script>

<div class="wrap">
  <div class="bar">
    <div class="fill" style="width: {progress.percent}%"></div>
  </div>
  <div class="msg" class:error={progress.stage === "error"}>
    {progress.message || (progress.stage === "idle" ? "等待操作" : "")}
  </div>
</div>

<style>
  .wrap {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .bar {
    height: 10px;
    background: #eef0f3;
    border-radius: 5px;
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: #4a7cff;
    transition: width 0.3s ease;
  }
  .msg {
    font-size: 13px;
    color: #555;
  }
  .msg.error {
    color: #c62828;
  }
</style>

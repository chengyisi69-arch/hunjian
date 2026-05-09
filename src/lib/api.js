import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export async function addScene(name) {
  return invoke("add_scene", { name });
}

export async function removeScene(name) {
  return invoke("remove_scene", { name });
}

export async function importAssets(sceneId, paths) {
  return invoke("import_assets", { sceneId, paths });
}

export async function preprocess(clipDurationSecs, orientation) {
  return invoke("preprocess", { clipDurationSecs, orientation });
}

export async function generateMix(params) {
  return invoke("generate_mix", { params });
}

export async function resetClips() {
  return invoke("reset_clips");
}

export async function getProject() {
  return invoke("get_project");
}

export async function saveProject(path) {
  return invoke("save_project", { path });
}

export async function loadProject(path) {
  return invoke("load_project", { path });
}

export async function newProject() {
  return invoke("new_project");
}

export async function exportVideo(src, dst) {
  return invoke("export_video", { src, dst });
}

// ── 模板 API ──

export async function saveTemplate(template) {
  return invoke("save_template", { template });
}

export async function listTemplates() {
  return invoke("list_templates");
}

export async function deleteTemplate(name) {
  return invoke("delete_template", { name });
}

export async function applyTemplate(name) {
  return invoke("apply_template", { name });
}

// ── AI API ──

export async function getAiConfig() {
  return invoke("get_ai_config");
}

export async function setAiConfig(config) {
  return invoke("set_ai_config", { config });
}

export async function aiGenerateTitle(description) {
  return invoke("ai_generate_title", { description });
}

export async function aiGenerateScript(description) {
  return invoke("ai_generate_script", { description });
}

export async function aiGenerateImage(prompt) {
  return invoke("ai_generate_image", { prompt });
}

export async function imageToVideo(imagePath, duration, effect, orientation) {
  return invoke("image_to_video", {
    imagePath,
    duration,
    effect,
    orientation,
  });
}

export async function addImageAsAsset(sceneId, imagePath, duration, orientation) {
  return invoke("add_image_as_asset", {
    sceneId,
    imagePath,
    duration,
    orientation,
  });
}

/** 注册 progress 事件监听，回调 ({ stage, percent, message }) */
export async function onProgress(cb) {
  return listen("progress", (e) => cb(e.payload));
}

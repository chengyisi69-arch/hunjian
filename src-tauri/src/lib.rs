mod audio;
mod commands;
mod compose;
mod error;
mod ffmpeg;
mod overlay;
mod pipeline;
mod preprocess;
mod selection;
mod state;
mod tts;
mod types;

mod ai;
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::add_scene,
            commands::remove_scene,
            commands::import_assets,
            commands::preprocess,
            commands::generate_mix,
            commands::reset_clips,
            commands::get_project,
            commands::save_project,
            commands::load_project,
            commands::new_project,
            commands::export_video,
            commands::save_template,
            commands::list_templates,
            commands::delete_template,
            commands::apply_template,
            commands::get_ai_config,
            commands::set_ai_config,
            commands::ai_generate_title,
            commands::ai_generate_script,
            commands::ai_generate_image,
            commands::image_to_video,
            commands::add_image_as_asset,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

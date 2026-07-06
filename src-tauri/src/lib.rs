mod commands;
mod config;
mod db;
mod llm;
mod model;
mod service;

use std::sync::Arc;

use commands::AppState;
use config::{build_llm, AppConfig};
use db::sqlite::SqliteStore;
use service::Service;
use tauri::Manager;
use tokio::sync::{Mutex, RwLock};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            let data_dir = app.path().app_data_dir().expect("app_data_dir");
            let db_path = data_dir.join("learning.db");
            let config_path = data_dir.join("config.json");

            let store = tauri::async_runtime::block_on(SqliteStore::open(&db_path))
                .expect("open sqlite");
            let cfg = AppConfig::load(&config_path).unwrap_or_default();
            let llm = build_llm(&cfg.llm);
            let service = Arc::new(Service {
                store: Arc::new(store),
                llm: RwLock::new(llm),
            });
            app.manage(AppState {
                service,
                config: Mutex::new(cfg),
                config_path,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::generate_starter_graph,
            commands::get_graph,
            commands::list_root_graphs,
            commands::list_nodes,
            commands::zoom_into_node,
            commands::create_node,
            commands::update_node_meta,
            commands::update_node_status,
            commands::delete_node,
            commands::request_verification,
            commands::submit_answer,
            commands::create_learning_script,
            commands::get_llm_config,
            commands::set_llm_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

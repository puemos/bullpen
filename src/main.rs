use bullpen::commands;
use bullpen::infra::acp::analysis_mcp_server;
use bullpen::state::AppState;

fn main() {
    env_logger::init();

    if std::env::args().any(|arg| arg == "--printenv") {
        bullpen::infra::shell::print_env_for_capture();
        return;
    }

    let _ = fix_path_env::fix();

    if std::env::args().any(|arg| arg == "--analysis-mcp-server") {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build MCP runtime");
        runtime
            .block_on(analysis_mcp_server::run_analysis_mcp_server())
            .expect("analysis MCP server failed");
        return;
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::get_agents,
            commands::get_settings,
            commands::update_settings,
            commands::get_all_analyses,
            commands::get_analysis_report,
            commands::delete_analysis,
            commands::create_analysis,
            commands::generate_analysis,
            commands::stop_analysis,
            commands::set_active_run,
            commands::get_run_progress,
            commands::export_analysis_markdown,
            commands::export_analysis_html,
            commands::publish_analysis_html,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Bullpen");
}

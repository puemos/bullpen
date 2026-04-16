use crazylines::commands;
use crazylines::infra::acp::analysis_mcp_server;
use crazylines::state::AppState;

fn main() {
    env_logger::init();

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running Crazylines");
}

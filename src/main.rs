use bullpen::commands;
use bullpen::infra::acp::analysis_mcp_server;
use bullpen::state::AppState;

fn main() {
    let _ = env_logger::try_init();

    if std::env::args().any(|arg| arg == "--printenv") {
        bullpen::infra::shell::print_env_for_capture();
        return;
    }

    let _ = fix_path_env::fix();

    // Capture the user's interactive PATH once, on the main thread, before
    // any worker threads or the Tauri runtime spawn. Later, ACP child
    // processes may also call this, but at that point the value is cached
    // behind a `OnceLock` so no `env::set_var` races are possible.
    bullpen::infra::shell::init_process_path();

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

    let state = match AppState::try_new() {
        Ok(state) => state,
        Err(err) => {
            eprintln!("Failed to open Bullpen database: {err:#}");
            std::process::exit(1);
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::get_agents,
            commands::get_settings,
            commands::update_settings,
            commands::get_all_analyses,
            commands::get_analysis_report,
            commands::get_stance_stale_metrics,
            commands::delete_analysis,
            commands::create_portfolio,
            commands::get_portfolios,
            commands::get_portfolio_detail,
            commands::import_portfolio_csv,
            commands::delete_portfolio,
            commands::rename_portfolio,
            commands::create_analysis,
            commands::generate_analysis,
            commands::stop_analysis,
            commands::get_price_history,
            commands::set_active_run,
            commands::get_run_progress,
            commands::export_analysis_markdown,
            commands::export_analysis_html,
            commands::publish_analysis_html,
            commands::list_sources,
            commands::refresh_source_key_status,
            commands::set_source_key,
            commands::clear_source_key,
            commands::test_source_key,
            commands::set_enabled_sources,
            commands::update::get_app_version,
            commands::update::run_self_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Bullpen");
}

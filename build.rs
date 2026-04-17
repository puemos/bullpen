use std::fs;
use std::path::Path;

fn main() {
    // The `export_analysis_html` command embeds a single-file viewer build
    // via `include_str!`. If the frontend hasn't been built yet (e.g. a plain
    // `cargo check` in a fresh clone), write a minimal placeholder so the
    // Rust crate compiles. Real builds go through Tauri's beforeBuildCommand,
    // which runs `pnpm build` and produces the real template.
    let target = Path::new("frontend/dist-viewer/viewer.html");
    if !target.exists() {
        if let Some(parent) = target.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let placeholder = "<!doctype html><meta charset=\"utf-8\"><title>Bullpen report</title>\
             <script>window.__BULLPEN_REPORT__ = \"__BULLPEN_REPORT_JSON__\";\
             document.body && (document.body.textContent = \
             'Viewer template was not built. Run `pnpm build:viewer` in frontend/.');\
             </script>";
        let _ = fs::write(target, placeholder);
    }
    println!("cargo:rerun-if-changed=frontend/dist-viewer/viewer.html");
    tauri_build::build();
}

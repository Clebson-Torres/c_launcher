#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::{Manager, WindowEvent, tray::{TrayIconBuilder, TrayIconEvent}};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

#[cfg(desktop)]
use tauri_plugin_autostart::ManagerExt; 


#[derive(Clone, serde::Serialize)]
pub struct FileResult {
    pub name: String,
    pub path: String,
    pub is_app: bool,
    pub score: i64,
}

impl FileResult {
    fn new(name: String, path: String, is_app: bool, score: i64) -> Self {
        Self { name, path, is_app, score }
    }
}

pub struct AppState {
    pub hotkey_registered: Mutex<bool>,
}

// ============================================
// 2. MÓDULO DE COMANDOS (Isolamento)
// ============================================
// Isso previne o erro "defined multiple times"
pub mod commands {
    use super::*; // Permite acessar FileResult, AppState, etc.
    use directories::UserDirs;
    use fuzzy_matcher::skim::SkimMatcherV2;
    use fuzzy_matcher::FuzzyMatcher;
    use std::process::Command;
    use walkdir::WalkDir;

    #[tauri::command]
    pub async fn search_files(query: String) -> Result<Vec<FileResult>, String> {
        let mut results = Vec::new();
        let query_trim = query.trim();
    
        if query_trim.is_empty() {
            return Ok(get_common_apps());
        }
    
        // 1. Comandos Web
        if let Some(web_res) = handle_web_prefixes(query_trim) {
            results.push(web_res);
            return Ok(results);
        }
    
        // 2. Calculadora
        if let Some(calc_res) = handle_calculator(query_trim) {
            results.push(calc_res);
            return Ok(results);
        }
    
        let matcher = SkimMatcherV2::default();
        let query_lower = query_trim.to_lowercase();
        
        if query_trim.starts_with('>') {
            let cmd_text = query_trim.strip_prefix('>').unwrap_or("").trim();
                if !cmd_text.is_empty() {
                    results.push(FileResult::new(
                        format!("💻 Executar: {}", cmd_text),
                        format!("terminal:{}", cmd_text), // Prefixo interno para o open_file
                        true,
                        20000 // Prioridade máxima
                    ));
                    return Ok(results);
                }
}
        // 3. Busca de Arquivos
        if let Some(user_dirs) = UserDirs::new() {
            let search_dirs = vec![
                user_dirs.desktop_dir(),
                user_dirs.document_dir(),
                user_dirs.download_dir(),
            ];
            for dir in search_dirs.into_iter().flatten() {
                search_in_directory(dir, &query_lower, &matcher, &mut results);
            }
        }
    
        // 4. Executáveis
        search_executables(&query_lower, &matcher, &mut results);
    
        // 5. Apps Comuns
        for app in get_common_apps() {
            if matcher.fuzzy_match(&app.name.to_lowercase(), &query_lower).is_some() {
                results.push(app);
            }
        }
    
        results.sort_by(|a, b| b.score.cmp(&a.score));
        results.truncate(20);
        Ok(results)
    }

    #[tauri::command]
    pub async fn open_file(path: String) -> Result<(), String> {
        if path.starts_with("result:") { return Ok(()); }

        if path.starts_with("terminal:") {
            let _cmd = path.strip_prefix("terminal:").unwrap_or("");

            if path.starts_with("terminal:") {
        let cmd = path.strip_prefix("terminal:").unwrap_or("");
        
        #[cfg(target_os = "windows")]
        {
            
            std::process::Command::new("cmd")
                .args(["/C", "start", "powershell", "-NoExit", "-Command", cmd])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        return Ok(());

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            if path.starts_with("terminal:") {
                let cmd = path.strip_prefix("terminal:").unwrap();
                // No Linux usamos x-terminal-emulator, no Mac usamos o comando 'open'
                let shell_cmd = if cfg!(target_os = "macos") {
                    format!("osascript -e 'tell application \"Terminal\" to do script \"{}\"'", cmd)
                } else {
                    format!("x-terminal-emulator -e {}", cmd)
                };

                std::process::Command::new("sh")
                    .arg("-c")
                    .arg(shell_cmd)
                    .spawn().map_err(|e| e.to_string())?;
                return Ok(());
            }
        }
    }
}
            
            

    
        #[cfg(target_os = "windows")]
        {
            if path.starts_with("http") || path.starts_with("mailto:") || path.starts_with("ms-settings:") {
                Command::new("cmd").args(["/C", "start", "", &path]).spawn().map_err(|e| e.to_string())?;
            } else {
                open::that(&path).map_err(|e| e.to_string())?;
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            open::that(&path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    #[tauri::command]
    pub async fn hide_window(window: tauri::Window) -> Result<(), String> {
        window.hide().map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub async fn show_window(window: tauri::Window) -> Result<(), String> {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())
    }

    // --- Helpers internos do módulo commands ---

    fn handle_web_prefixes(query: &str) -> Option<FileResult> {
        let prefixes = [
            ("g:", "google:", "🔍 Google", "https://www.google.com/search?q="),
            ("yt:", "youtube:", "▶️ YouTube", "https://www.youtube.com/results?search_query="),
            ("gh:", "github:", "🐙 GitHub", "https://github.com/search?q="),
            ("wiki:", "wiki:", "📖 Wikipedia", "https://pt.wikipedia.org/wiki/"),
        ];
    
        for (s, l, label, url) in prefixes {
            if query.starts_with(s) || query.starts_with(l) {
                let term = query.split(':').nth(1).unwrap_or("").trim();
                return Some(FileResult::new(
                    format!("{} : {}", label, term),
                    format!("{}{}", url, urlencoding::encode(term)),
                    true, 10000
                ));
            }
        }
        None
    }
    
    fn handle_calculator(query: &str) -> Option<FileResult> {
        let clean = query.replace("calc:", "").replace("=", "").trim().to_string();
        if clean.chars().any(|c| c.is_numeric()) && clean.chars().any(|c| "+-*/^(".contains(c)) {
            if let Ok(res) = meval::eval_str(clean.replace('x', "*").replace(',', ".")) {
                return Some(FileResult::new(format!("🧮 = {}", res), format!("result:{}", res), true, 10000));
            }
        }
        None
    }
    
    fn search_in_directory(dir: &std::path::Path, query: &str, matcher: &SkimMatcherV2, results: &mut Vec<FileResult>) {
        for entry in WalkDir::new(dir).max_depth(3).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
            let name = entry.file_name().to_string_lossy();
            if let Some(score) = matcher.fuzzy_match(&name.to_lowercase(), query) {
                results.push(FileResult::new(name.to_string(), entry.path().display().to_string(), false, score));
            }
        }
    }
    
    fn search_executables(query: &str, matcher: &SkimMatcherV2, results: &mut Vec<FileResult>) {
        #[cfg(target_os = "windows")]
        {
            let paths = ["C:\\Program Files", "C:\\Program Files (x86)"];
            for path in paths.iter().map(std::path::Path::new).filter(|p| p.exists()) {
                for entry in WalkDir::new(path).max_depth(2).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
                    if entry.path().extension().map_or(false, |ext| ext == "exe") {
                        let name = entry.file_name().to_string_lossy();
                        if let Some(score) = matcher.fuzzy_match(&name.to_lowercase(), query) {
                            results.push(FileResult::new(name.to_string(), entry.path().display().to_string(), true, score + 2000));
                        }
                    }
                }
            }
        }
    }
    
    fn get_common_apps() -> Vec<FileResult> {
        vec![
            FileResult::new("Calculadora".into(), "calc.exe".into(), true, 1000),
            FileResult::new("Bloco de Notas".into(), "notepad.exe".into(), true, 900),
            FileResult::new("VS Code".into(), "code".into(), true, 800),
        ]
    }
}

// ============================================
// 3. ENTRY POINT (RUN)
// ============================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--hidden"])))
        .plugin(tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, s, event| {
                if s == &shortcut && event.state == ShortcutState::Pressed {
                    if let Some(window) = app.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) { let _ = window.hide(); }
                        else { let _ = window.show(); let _ = window.set_focus(); }
                    }
                }
            }).build())
        .manage(AppState { hotkey_registered: Mutex::new(false) })
        // AQUI: Usamos commands::nome_da_funcao para evitar o erro de reimport
        .invoke_handler(tauri::generate_handler![
            commands::search_files, 
            commands::open_file, 
            commands::hide_window, 
            commands::show_window
        ])
        .setup(move |app| {

            app.global_shortcut().register(shortcut).expect("Erro ao registrar atalho");

            #[cfg(desktop)]
            {
                let autostart_manager = app.autolaunch(); 
                let _ = autostart_manager.enable(); 
                println!("Registrado para autostart? {}", autostart_manager.is_enabled().unwrap_or(false));
            }

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("CLauncher")
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("Erro ao iniciar aplicação");
}
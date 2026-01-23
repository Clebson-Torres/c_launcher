#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::{Manager, tray::{TrayIconBuilder, TrayIconEvent}};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

#[cfg(target_os = "windows")]
use tauri_plugin_autostart::ManagerExt;


#[derive(Clone, serde::Serialize)]
pub struct ClipboardItem {
    pub content: String,
    pub is_sensitive: bool,
}

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
    pub clipboard_history: Mutex<Vec<ClipboardItem>>,
}



fn is_likely_sensitive(text: &str) -> bool {
    let t = text.trim();
    if t.is_empty() { return false; }
    if t.starts_with("ghp_") || t.starts_with("sk-") { return true; }
    let has_number = t.chars().any(|c| c.is_numeric());
    let has_special = t.chars().any(|c| !c.is_alphanumeric());
    if t.len() < 35 && has_number && has_special { return true; }
    false
}

pub mod commands {
    use super::*;
    use directories::UserDirs;
    use fuzzy_matcher::skim::SkimMatcherV2;
    use fuzzy_matcher::FuzzyMatcher;
    use std::process::Command;
    use walkdir::WalkDir;

    #[tauri::command]
    pub async fn search_files(app_handle: tauri::AppHandle, query: String) -> Result<Vec<FileResult>, String> {
        let mut results = Vec::new();
        let query_trim = query.trim();
        let state = app_handle.state::<AppState>();

       
        if query_trim.starts_with("clip:") {
            let history = state.clipboard_history.lock().unwrap();
            let search_term = query_trim.strip_prefix("clip:").unwrap_or("").to_lowercase();

            for item in history.iter() {
                let display_name = if item.is_sensitive {
                    "🔒 [Conteúdo Sensível Oculto]".to_string()
                } else {
                    let preview: String = item.content.chars().take(45).collect();
                    format!("📋 {}", if preview.len() < item.content.len() { format!("{}...", preview) } else { preview })
                };

                if search_term.is_empty() || item.content.to_lowercase().contains(&search_term) {
                    results.push(FileResult::new(display_name, format!("clip:{}", item.content), true, 15000));
                }
            }
            return Ok(results);
        }

        
        if query_trim.is_empty() { return Ok(get_common_apps()); }

        let matcher = SkimMatcherV2::default();
        let query_lower = query_trim.to_lowercase();

        if query_trim.starts_with('>') {
            let cmd_text = query_trim.strip_prefix('>').unwrap_or("").trim();
            if !cmd_text.is_empty() {
                results.push(FileResult::new(format!("💻 Executar: {}", cmd_text), format!("terminal:{}", cmd_text), true, 20000));
                return Ok(results);
            }
        }

        if let Some(user_dirs) = UserDirs::new() {
            let search_dirs = vec![user_dirs.desktop_dir(), user_dirs.document_dir(), user_dirs.download_dir()];
            for dir in search_dirs.into_iter().flatten() {
                for entry in WalkDir::new(dir).max_depth(3).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
                    let name = entry.file_name().to_string_lossy();
                    if let Some(score) = matcher.fuzzy_match(&name.to_lowercase(), &query_lower) {
                        results.push(FileResult::new(name.to_string(), entry.path().display().to_string(), false, score));
                    }
                }
            }
        }

        
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
        if path.starts_with("clip:") {
            let content = path.strip_prefix("clip:").unwrap_or("");
            let mut clipboard = arboard::Clipboard::new().map_err(|e: arboard::Error| e.to_string())?;
            return clipboard.set_text(content.to_string()).map_err(|e: arboard::Error| e.to_string());
        }

        if path.starts_with("terminal:") {
            let cmd = path.strip_prefix("terminal:").unwrap_or("");
            #[cfg(target_os = "windows")]
            {
                Command::new("cmd").args(["/C", "start", "powershell", "-NoExit", "-Command", cmd]).spawn().map_err(|e| e.to_string())?;
            }
            return Ok(());
        }

        open::that(&path).map_err(|e| e.to_string())
    }

    pub fn get_common_apps() -> Vec<FileResult> {
        #[cfg(target_os = "windows")]
        {
            vec![
                FileResult::new("Bloco de Notas".into(), "notepad.exe".into(), true, 1000),
                FileResult::new("Prompt de Comando".into(), "cmd.exe".into(), true, 950),
                FileResult::new("Gerenciador de Tarefas".into(), "taskmgr.exe".into(), true, 900),
                FileResult::new("Configurações".into(), "ms-settings:home".into(), true, 850),
                FileResult::new("Explorador de Arquivos".into(), "explorer.exe".into(), true, 800),                
                FileResult::new("Painel de Controle".into(), "control.exe".into(), true, 700),
            ]
        }
        #[cfg(not(target_os = "windows"))]
        {
            vec![
                FileResult::new("Terminal".into(), "x-terminal-emulator".into(), true, 1000),
                FileResult::new("Editor de Texto".into(), "gedit".into(), true, 900),
            ]
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space);
    let mut builder = tauri::Builder::default();

    #[cfg(target_os = "windows")]
    {
        builder = builder.plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--hidden"])));
    }

    builder
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, s, event| {
                if s == &shortcut && event.state == ShortcutState::Pressed {
                    if let Some(window) = app.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) { let _ = window.hide(); }
                        else { let _ = window.show(); let _ = window.set_focus(); }
                    }
                }
            }).build())
        .manage(AppState { 
            hotkey_registered: Mutex::new(false),
            clipboard_history: Mutex::new(Vec::new()),
        })
        .invoke_handler(tauri::generate_handler![
            commands::search_files, 
            commands::open_file
        ])
        .setup(move |app| {
            app.global_shortcut().register(shortcut).expect("Erro atalho");
            let handle = app.handle().clone();

            
            std::thread::spawn(move || {
                let mut clipboard = arboard::Clipboard::new().expect("Falha clipboard");
                let mut last_content = String::new();
                loop {
                    if let Ok(text_val) = clipboard.get_text() {
                        let text: String = text_val.trim().to_string();
                        if !text.is_empty() && text != last_content {
                            last_content = text.clone();
                            let state = handle.state::<AppState>();
                            let mut history = state.clipboard_history.lock().unwrap();
                            let sensitive = is_likely_sensitive(&text);
                            history.retain(|x| x.content != text);
                            history.insert(0, ClipboardItem { content: text, is_sensitive: sensitive });
                            if history.len() > 30 { history.pop(); }
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(800));
                }
            });

            #[cfg(target_os = "windows")]
            { let _ = app.autolaunch().enable(); }

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                        let _ = tray.app_handle().get_webview_window("main").map(|w| { let _ = w.show(); let _ = w.set_focus(); });
                    }
                })
                .build(app)?;

            Ok(())
        })
        
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::Focused(focused) => {
                    if !focused {
                        let _ = window.hide();
                    }
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("Erro ao iniciar");
}
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use directories::UserDirs;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::process::Command;
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tauri_plugin_shell;
use walkdir::WalkDir;

#[derive(Clone, serde::Serialize)]
struct FileResult {
    name: String,
    path: String,
    is_app: bool,
    score: i64,
}

impl FileResult {
    fn new(name: String, path: String, is_app: bool, score: i64) -> Self {
        Self {
            name,
            path,
            is_app,
            score,
        }
    }
}

struct AppState {
    hotkey_registered: Mutex<bool>,
}

#[tauri::command]
async fn search_files(query: String) -> Result<Vec<FileResult>, String> {
    let mut results = Vec::new();

    if query.trim().is_empty() {
        return Ok(get_common_apps());
    }

    // ============================================
    // COMANDOS CUSTOMIZADOS - DETECTAR PREFIXOS
    // ============================================
    
    // 1. GOOGLE: google: termo ou g: termo
    if query.starts_with("google:") || query.starts_with("g:") {
        let search_term = query.replace("google:", "").replace("g:", "").trim().to_string();
        let url = format!("https://www.google.com/search?q={}", 
            urlencoding::encode(&search_term));
        
        results.push(FileResult::new(
            format!("🔍 Buscar '{}' no Google", search_term),
            url,
            true,
            10000,
        ));
        return Ok(results);
    }

    // 2. YOUTUBE: yt: termo ou youtube: termo
    if query.starts_with("yt:") || query.starts_with("youtube:") {
        let search_term = query.replace("yt:", "").replace("youtube:", "").trim().to_string();
        let url = format!("https://www.youtube.com/results?search_query={}", 
            urlencoding::encode(&search_term));
        
        results.push(FileResult::new(
            format!("▶️ Buscar '{}' no YouTube", search_term),
            url,
            true,
            10000,
        ));
        return Ok(results);
    }

    // 3. CALCULADORA: calc: expressão ou = expressão ou apenas números/operadores
    if query.starts_with("calc:") || query.starts_with("=") {
        let expression = query.replace("calc:", "").replace("=", "").trim().to_string();
        if let Ok(result) = evaluate_math(&expression) {
            results.push(FileResult::new(
                format!("🧮 {} = {}", expression, result),
                format!("result:{}", result),
                true,
                10000,
            ));
            return Ok(results);
        }
    }

    // Auto-detectar expressões matemáticas (sem prefixo)
    if is_math_expression(&query) {
        if let Ok(result) = evaluate_math(&query) {
            results.push(FileResult::new(
                format!("🧮 {} = {}", query, result),
                format!("result:{}", result),
                true,
                10000,
            ));
        }
    }

    // 4. TRADUÇÃO: tr: texto ou translate: texto
    if query.starts_with("tr:") || query.starts_with("translate:") {
        let text = query.replace("tr:", "").replace("translate:", "").trim().to_string();
        let url = format!("https://translate.google.com/?sl=auto&tl=pt&text={}", 
            urlencoding::encode(&text));
        
        results.push(FileResult::new(
            format!("🌐 Traduzir '{}'", text),
            url,
            true,
            10000,
        ));
        return Ok(results);
    }

    // 5. GITHUB: gh: repo ou github: repo
    if query.starts_with("gh:") || query.starts_with("github:") {
        let repo = query.replace("gh:", "").replace("github:", "").trim().to_string();
        let url = if repo.contains('/') {
            format!("https://github.com/{}", repo)
        } else {
            format!("https://github.com/search?q={}", urlencoding::encode(&repo))
        };
        
        results.push(FileResult::new(
            format!("🐙 GitHub: {}", repo),
            url,
            true,
            10000,
        ));
        return Ok(results);
    }

    // 6. REDDIT: reddit: termo ou r: termo
    if query.starts_with("reddit:") || query.starts_with("r:") {
        let term = query.replace("reddit:", "").replace("r:", "").trim().to_string();
        let url = if term.starts_with('/') {
            format!("https://reddit.com{}", term)
        } else {
            format!("https://www.reddit.com/search/?q={}", urlencoding::encode(&term))
        };
        
        results.push(FileResult::new(
            format!("🤖 Reddit: {}", term),
            url,
            true,
            10000,
        ));
        return Ok(results);
    }

    // 7. STACK OVERFLOW: so: termo ou stack: termo
    if query.starts_with("so:") || query.starts_with("stack:") {
        let term = query.replace("so:", "").replace("stack:", "").trim().to_string();
        let url = format!("https://stackoverflow.com/search?q={}", 
            urlencoding::encode(&term));
        
        results.push(FileResult::new(
            format!("📚 Stack Overflow: {}", term),
            url,
            true,
            10000,
        ));
        return Ok(results);
    }

    // 8. WIKIPEDIA: wiki: termo
    if query.starts_with("wiki:") {
        let term = query.replace("wiki:", "").trim().to_string();
        let url = format!("https://pt.wikipedia.org/wiki/{}", 
            urlencoding::encode(&term.replace(" ", "_")));
        
        results.push(FileResult::new(
            format!("📖 Wikipedia: {}", term),
            url,
            true,
            10000,
        ));
        return Ok(results);
    }

    // 9. AMAZON: amazon: produto
    if query.starts_with("amazon:") {
        let product = query.replace("amazon:", "").trim().to_string();
        let url = format!("https://www.amazon.com.br/s?k={}", 
            urlencoding::encode(&product));
        
        results.push(FileResult::new(
            format!("🛒 Amazon: {}", product),
            url,
            true,
            10000,
        ));
        return Ok(results);
    }

    // 10. MAPS: maps: local ou m: local
    if query.starts_with("maps:") || query.starts_with("m:") {
        let location = query.replace("maps:", "").replace("m:", "").trim().to_string();
        let url = format!("https://www.google.com/maps/search/{}", 
            urlencoding::encode(&location));
        
        results.push(FileResult::new(
            format!("🗺️ Maps: {}", location),
            url,
            true,
            10000,
        ));
        return Ok(results);
    }

    // 11. EMAIL: mail: destinatário ou email: destinatário
    if query.starts_with("mail:") || query.starts_with("email:") {
        let email = query.replace("mail:", "").replace("email:", "").trim().to_string();
        let url = format!("mailto:{}", email);
        
        results.push(FileResult::new(
            format!("📧 Email para: {}", email),
            url,
            true,
            10000,
        ));
        return Ok(results);
    }

    // 12. CONVERSÃO: 100 usd para brl
    if query.contains(" para ") || query.contains(" to ") {
        if let Some((from, to)) = parse_conversion(&query) {
            let url = format!("https://www.google.com/search?q={}", 
                urlencoding::encode(&format!("{} to {}", from, to)));
            
            results.push(FileResult::new(
                format!("💱 Converter {} para {}", from, to),
                url,
                true,
                10000,
            ));
            return Ok(results);
        }
    }

    // Se não for nenhum comando, fazer busca normal de arquivos
    let matcher = SkimMatcherV2::default();
    let query_lower = query.to_lowercase();

    if let Some(user_dirs) = UserDirs::new() {
        let search_dirs = vec![
            user_dirs.desktop_dir(),
            user_dirs.document_dir(),
            user_dirs.download_dir(),
        ];

        for dir_opt in search_dirs {
            if let Some(dir) = dir_opt {
                search_in_directory(&dir, &query_lower, &matcher, &mut results);
            }
        }
    }

    if let Ok(current_dir) = std::env::current_dir() {
        search_in_directory(&current_dir, &query_lower, &matcher, &mut results);
    }

    search_executables(&query_lower, &matcher, &mut results);

    let common_apps = get_common_apps();
    for app in common_apps {
        if matcher
            .fuzzy_match(&app.name.to_lowercase(), &query_lower)
            .is_some()
        {
            results.push(app);
        }
    }

    results.sort_by(|a, b| b.score.cmp(&a.score));
    results.truncate(20);

    Ok(results)
}

// ============================================
// FUNÇÕES AUXILIARES PARA COMANDOS
// ============================================

fn is_math_expression(s: &str) -> bool {
    let s = s.trim();
    let has_number = s.chars().any(|c| c.is_numeric());
    let has_operator = s.contains('+') || s.contains('-') || s.contains('*') 
        || s.contains('/') || s.contains('^') || s.contains('(');
    has_number && has_operator
}

fn evaluate_math(expression: &str) -> Result<f64, String> {
    use meval::eval_str;
    
    let clean = expression
        .replace("x", "*")
        .replace("×", "*")
        .replace("÷", "/")
        .replace(",", ".");
    
    eval_str(&clean).map_err(|e| e.to_string())
}

fn parse_conversion(query: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = if query.contains(" para ") {
        query.split(" para ").collect()
    } else {
        query.split(" to ").collect()
    };
    
    if parts.len() == 2 {
        Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
    } else {
        None
    }
}

// ============================================
// FUNÇÕES ORIGINAIS DE BUSCA
// ============================================

fn search_in_directory(
    dir: &std::path::Path,
    query: &str,
    matcher: &SkimMatcherV2,
    results: &mut Vec<FileResult>,
) {
    for entry in WalkDir::new(dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy();
        let name_lower = name.to_lowercase();

        if let Some(score) = matcher.fuzzy_match(&name_lower, query) {
            let bonus = if path.extension().map_or(false, |ext| {
                let ext_str = ext.to_string_lossy().to_lowercase();
                ext_str == "exe" || ext_str == "lnk" || ext_str == "app"
            }) {
                1000
            } else {
                0
            };

            results.push(FileResult::new(
                name.to_string(),
                path.display().to_string(),
                false,
                score + bonus,
            ));
        }

        if results.len() >= 50 {
            break;
        }
    }
}

fn search_executables(query: &str, matcher: &SkimMatcherV2, results: &mut Vec<FileResult>) {
    #[cfg(target_os = "windows")]
    {
        let program_dirs = vec![
            std::path::Path::new("C:\\Program Files"),
            std::path::Path::new("C:\\Program Files (x86)"),
        ];

        for dir in program_dirs {
            if dir.exists() {
                for entry in WalkDir::new(dir)
                    .max_depth(2)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext.to_string_lossy().to_lowercase() == "exe" {
                            let name = entry.file_name().to_string_lossy();
                            let name_lower = name.to_lowercase();

                            if let Some(score) = matcher.fuzzy_match(&name_lower, query) {
                                results.push(FileResult::new(
                                    name.to_string(),
                                    path.display().to_string(),
                                    true,
                                    score + 2000,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
}

fn get_common_apps() -> Vec<FileResult> {
    vec![
        FileResult::new(
            "Calculadora".to_string(),
            "calc.exe".to_string(),
            true,
            1000,
        ),
        FileResult::new(
            "Bloco de Notas".to_string(),
            "notepad.exe".to_string(),
            true,
            900,
        ),
        FileResult::new(
            "VS Code".to_string(),
            "C:\\Users\\clebs\\AppData\\Local\\Programs\\Microsoft VS Code\\Code.exe".to_string(),
            true,
            800,
        ),
        FileResult::new(
            "PowerShell".to_string(),
            "powershell.exe".to_string(),
            true,
            700,
        ),
        FileResult::new(
            "Explorador de Arquivos".to_string(),
            "explorer.exe".to_string(),
            true,
            600,
        ),
        FileResult::new(
            "Prompt de Comando".to_string(),
            "cmd.exe".to_string(),
            true,
            500,
        ),
        FileResult::new(
            "Configurações".to_string(),
            "ms-settings:".to_string(),
            true,
            400,
        ),
        FileResult::new(
            "Painel de Controle".to_string(),
            "control.exe".to_string(),
            true,
            300,
        ),
        FileResult::new(
            "Gerenciador de Tarefas".to_string(),
            "taskmgr.exe".to_string(),
            true,
            200,
        ),
    ]
}

#[tauri::command]
async fn open_file(path: String) -> Result<(), String> {
    // Detectar se é resultado de calculadora
    if path.starts_with("result:") {
        // Apenas copiar para clipboard (opcional)
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        if path.starts_with("http://") || path.starts_with("https://") || path.starts_with("mailto:") {
            let _ = Command::new("cmd")
                .args(["/C", "start", "", &path])
                .spawn()
                .map_err(|e| e.to_string())?;
        } else if path.starts_with("ms-settings:") {
            let _ = Command::new("cmd")
                .args(["/C", "start", "", &path])
                .spawn()
                .map_err(|e| e.to_string())?;
        } else {
            let _ = open::that(&path).map_err(|e| e.to_string())?;
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = open::that(&path).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
async fn hide_window(window: tauri::Window) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())
}

#[tauri::command]
async fn show_window(window: tauri::Window) -> Result<(), String> {
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())
}

fn main() {
    let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space);
    tauri::Builder::default()
        .manage(AppState {
            hotkey_registered: Mutex::new(false),
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, s, event| {
                    if s == &shortcut && event.state == ShortcutState::Pressed {
                        let window = app.get_webview_window("main").unwrap();
                        if window.is_visible().unwrap() {
                            window.hide().unwrap();
                        } else {
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            search_files,
            open_file,
            hide_window,
            show_window
        ])
        .setup(move |app| {
            let state = app.state::<AppState>();
            let mut hotkey_registered = state.hotkey_registered.lock().unwrap();

            if !*hotkey_registered {
                app.global_shortcut().register(shortcut).unwrap();
                *hotkey_registered = true;
            }

            let window = app.get_webview_window("main").unwrap();
            
            // Remove título do localhost em dev
            #[cfg(debug_assertions)]
            window.set_title("").unwrap();
            
            window.hide().unwrap();

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
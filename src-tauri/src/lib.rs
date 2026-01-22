#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(|app| {

            let show = MenuItem::new(app, "Abrir", true, None::<&str>)?;
            let quit = MenuItem::new(app, "Sair", true, None::<&str>)?;

            let menu = Menu::new(app)?;
            menu.append(&show)?;
            menu.append(&quit)?;


            let tray_icon = match app.default_window_icon() {
                Some(icon) => icon.clone(),
                None => {
                    println!("⚠ Nenhum ícone padrão encontrado, usando fallback");
                    load_icon_from_png()
                }
            };
            

            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .menu(&menu)
                .tooltip("CLauncher")
                .on_menu_event(|app: &tauri::AppHandle, event| {
                    match event.id().as_ref() {
                        "Abrir" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "Sair" => {
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { .. } = event {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)
                .expect("Erro ao criar tray icon");

            println!("✓ Tray icon criado com sucesso!");

            // ===== ESCONDE AO INICIAR =====
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[allow(dead_code)]
fn load_icon_from_png() -> Image<'static> {
    let bytes = include_bytes!("../icons/cl.png");
    
    
    let img = image::load_from_memory(bytes)
        .expect("Falha ao decodificar PNG")
        .to_rgba8();
    
    let (width, height) = img.dimensions();
    let rgba_data = img.into_raw();
    
    Image::new_owned(rgba_data, width, height)
}
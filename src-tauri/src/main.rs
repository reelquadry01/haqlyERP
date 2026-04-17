// Author: Quadri Atharu
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use haqly_erp::{
    commands::{
        ai_commands, app_commands, einvoice_commands, ocr_commands,
    },
    sidecar::manager::SidecarManager,
};

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, RunEvent,
};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,haqly_erp=debug")),
        )
        .with_file(true)
        .with_line_number(true)
        .init();

    let sidecar_manager = SidecarManager::new();

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(sidecar_manager)
        .invoke_handler(tauri::generate_handler![
            app_commands::get_app_version,
            app_commands::open_external_link,
            app_commands::toggle_fullscreen,
            app_commands::get_system_info,
            app_commands::check_for_updates,
            app_commands::store_credential,
            app_commands::get_credential,
            app_commands::delete_credential,
            ocr_commands::process_document,
            ocr_commands::get_ocr_status,
            ocr_commands::get_document_history,
            ocr_commands::submit_for_review,
            einvoice_commands::validate_invoice_nrs,
            einvoice_commands::sign_invoice_nrs,
            einvoice_commands::confirm_invoice_nrs,
            einvoice_commands::download_invoice_nrs,
            einvoice_commands::get_einvoice_status,
            ai_commands::analyze_financials,
            ai_commands::compute_tax,
            ai_commands::generate_report,
            ai_commands::get_ai_status,
        ])
        .setup(|app| {
            let dashboard_item = MenuItemBuilder::with_id("dashboard", "Dashboard").build(app)?;
            let reports_item = MenuItemBuilder::with_id("reports", "Reports").build(app)?;
            let settings_item = MenuItemBuilder::with_id("settings", "Settings").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

            let menu = MenuBuilder::new(app)
                .items(&[&dashboard_item, &reports_item, &settings_item, &quit_item])
                .build()?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("HAQLY ERP")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "dashboard" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "reports" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.emit("navigate", "/reports");
                        }
                    }
                    "settings" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.emit("navigate", "/settings");
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Start the Axum server inside the Tauri process
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime for Axum");
                rt.block_on(async {
                    if let Err(e) = haqly_erp_server::start_server().await {
                        tracing::error!("Axum server error: {e}");
                    }
                });
            });

            // Start AI sidecar (non-fatal if it fails)
            let sidecar_mgr = app.state::<SidecarManager>();
            let mgr_clone = sidecar_mgr.inner().clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime for AI sidecar");
                rt.block_on(async {
                    if let Err(e) = mgr_clone.start_ai_engine().await {
                        tracing::warn!("AI engine startup failed (non-fatal): {e}");
                    }
                });
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| match event {
        RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
            let sidecar_mgr = app_handle.state::<SidecarManager>();
            let mgr = sidecar_mgr.inner().clone();
            let handle = app_handle.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime for shutdown");
                rt.block_on(async {
                    if let Err(e) = mgr.stop_all().await {
                        tracing::error!("Sidecar shutdown error: {e}");
                    }
                });
                handle.exit(0);
            });
        }
        RunEvent::Exit => {
            tracing::info!("HAQLY ERP shutting down");
        }
        _ => {}
    });
}

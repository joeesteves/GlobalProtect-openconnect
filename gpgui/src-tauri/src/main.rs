#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use auth::{SamlBinding, AuthWindow};
use env_logger::Env;
use gpcommon::{Client, ServerApiError, VpnStatus};
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_log::LogTarget;

mod auth;

#[tauri::command]
async fn vpn_status<'a>(client: State<'a, Arc<Client>>) -> Result<VpnStatus, ServerApiError> {
    client.status().await
}

#[tauri::command]
async fn vpn_connect<'a>(
    server: String,
    cookie: String,
    client: State<'a, Arc<Client>>,
) -> Result<(), ServerApiError> {
    client.connect(server, cookie).await
}

#[tauri::command]
async fn vpn_disconnect<'a>(client: State<'a, Arc<Client>>) -> Result<(), ServerApiError> {
    client.disconnect().await
}

#[tauri::command]
async fn saml_login(
    binding: SamlBinding,
    request: String,
    app_handle: AppHandle,
) -> tauri::Result<()> {
    let auth_window = AuthWindow::new(app_handle, binding, String::from("PAN GlobalProtect"));
    if let Err(err) = auth_window.process(request) {
        println!("Error processing auth window: {}", err);
        return Err(err);
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
struct StatusPayload {
    status: VpnStatus,
}

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(Client::default());
    let client_clone = client.clone();
    let app_handle = app.handle();

    tauri::async_runtime::spawn(async move {
        let _ = client_clone.subscribe_status(move |status| {
            let payload = StatusPayload { status };
            if let Err(err) = app_handle.emit_all("vpn-status-received", payload) {
                println!("Error emitting event: {}", err);
            }
        });

        // let _ = client_clone.run().await;
    });

    app.manage(client);
    Ok(())
}

fn main() {
    // env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets([
                    LogTarget::LogDir,
                    LogTarget::Stdout, /*LogTarget::Webview*/
                ])
                .build(),
        )
        .setup(setup)
        .invoke_handler(tauri::generate_handler![
            vpn_status,
            vpn_connect,
            vpn_disconnect,
            saml_login,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

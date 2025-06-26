use tauri::window::Color;
use tauri::{command, AppHandle, Emitter, Manager, Url, WebviewUrl, WebviewWindowBuilder};

use log::info;

#[command]
fn greet(name: &str) -> String {
    info!("Greet command invoked with name: {}", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[command]
fn create_window(app_handle: AppHandle) {
    info!("Creating a new window...");

    if app_handle.get_webview("screen").is_some() {
        info!("Window with label 'screen' already exists, skipping creation.");
        return;
    }

    let _window = WebviewWindowBuilder::new(
        &app_handle,
        "screen",
        WebviewUrl::External(
            Url::parse("https://google.com").expect("Invalid URL for the new window"),
        ),
    )
    .title("Screen")
    .background_color(Color(1, 1, 1, 255))
    .resizable(true)
    .visible(true)
    .inner_size(600.0, 800.0)
    .initialization_script(
        r#"
    window.addEventListener('click', () => {
        window.__TAURI_INTERNALS__.invoke('reset_timer');
    });
    window.addEventListener('mousemove', () => {
        window.__TAURI_INTERNALS__.invoke('reset_timer');
    });
    window.addEventListener('keyup', (e) => {
        window.__TAURI_INTERNALS__.invoke('reset_timer');
        window.__TAURI_INTERNALS__.invoke('keyup', {key: e.key});
    });
"#,
    )
    .build()
    .expect("Failed to create new window");

    info!(
        "New window created successfully with label: {}",
        _window.label()
    );

    let webview = _window
        .get_webview("screen")
        .expect("Failed to get webview for the new window");

    info!("Webview for the new window: {:?}", webview.url().unwrap());
}

#[command]
fn change_url(app_handle: AppHandle, url: String) {
    let webview = app_handle
        .get_webview("screen")
        .expect("Failed to get webview for the new window");

    webview
        .navigate(Url::parse(&url).expect("Invalid URL provided"))
        .expect("Failed to navigate webview to new URL");

    webview.window().set_title(
        &format!("{}", url),
    ).expect("Failed to set window title");

    info!("Webview URL changed successfully to: {}", url);

}

#[allow(dead_code)]
#[command]
fn reset_timer(app_handle: AppHandle) {
    app_handle
        .emit("reset-timer", ())
        .expect("Failed to emit reset-timer event");
}

#[allow(dead_code)]
#[command]
fn keyup(app_handle: AppHandle, key: String) {
    info!("Key up event for key: {}", key);
    app_handle
        .emit("keyup", key)
        .expect("Failed to emit keyup event");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_log::Builder::default().build())
        .invoke_handler(tauri::generate_handler![greet, create_window, reset_timer, change_url, keyup])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

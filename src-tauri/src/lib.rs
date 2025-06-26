use tauri::window::Color;
use tauri::{command, AppHandle, Emitter, Manager, Url, WebviewUrl, WebviewWindowBuilder};

use log::info;

#[command]
fn greet(name: &str) -> String {
    info!("Greet command invoked with name: {}", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[command]
fn create_window(app_handle: AppHandle, url_list: Vec<String>) {
    info!("Creating a new window with URL list: {:?}", url_list);
    use std::fmt::Write as _;

    info!("Creating a new window...");

    if app_handle.get_webview("screen").is_some() {
        info!("Window with label 'screen' already exists, skipping creation.");
        return;
    }

    // Generate HTML for the URL list as dots with hover expansion (vertical stack)
    let mut url_list_html = String::from(
        r##"
        <style>
        #url-list * {
            all: initial;
        }

        #url-list {
            all: initial;
            display: flex;
            flex-direction: column;
            gap: 8px;
        }
        #url-list .url-dot-container {
            position: relative;
            display: flex;
            align-items: center;
            height: 20px;
        }
        #url-list .url-dot {
            width: 14px;
            height: 14px;
            background: #fff;
            border-radius: 50%;
            display: inline-block;
            cursor: pointer;
            border: 2px solid #2196f3;
            transition: box-shadow 0.2s;
            position: relative;
            z-index: 1;
        }
        #url-list .url-dot-container .url-tooltip {
            max-width: 260px;
            color: #fff;
            background: rgba(0, 0, 0, 0.5);
            text-align: left;
            border-radius: 4px;
            padding: 6px 10px;
            margin-left: 12px;
            white-space: pre-line;
            font-size: 13px;
            opacity: 0;
            visibility: hidden;
            transition: opacity 0.2s, visibility 0.2s;
            position: relative;
            left: 0;
            top: 0;
            pointer-events: none;
            z-index: 2;
            display: inline-block;
        }
        #url-list .url-dot:hover + .url-tooltip,
        #url-list .url-dot:focus + .url-tooltip {
            opacity: 1;
            visibility: visible;
        }
        #url-list .url-dot:hover {
            box-shadow: 0 0 0 3px #2196f3aa;
        }
        </style>
        <div id="url-list" style='position:fixed;top:0;left:0;padding:8px;z-index:9999;font-size:14px;max-width:300px;overflow:auto;'>
    "##,
    );
    for url in &url_list {
        // Escape single quotes for JS/HTML safety
        let safe_url = url.replace('\'', "\\'");
        let _ = write!(
            url_list_html,
            r##"<div class="url-dot-container">
    <a href="#" class="url-dot" tabindex="0" onclick="window.__TAURI_INTERNALS__.invoke('change_url', {{ url: '{}' }}); return false;"></a>
    <span class="url-tooltip">{}</span>
</div>"##,
            safe_url, url
        );
    }
    url_list_html.push_str("</div>");

    // JS to inject the URL list HTML
    let inject_script = format!(
        r#"
        document.addEventListener('DOMContentLoaded', function() {{
            document.body.insertAdjacentHTML('afterbegin', `{}`);
        }});
        window.addEventListener('click', () => {{
            window.__TAURI_INTERNALS__.invoke('reset_timer');
        }});
        window.addEventListener('mousemove', () => {{
            window.__TAURI_INTERNALS__.invoke('reset_timer');
        }});
        window.addEventListener('keyup', (e) => {{
            window.__TAURI_INTERNALS__.invoke('reset_timer');
            window.__TAURI_INTERNALS__.invoke('keyup', {{key: e.key}});
        }});
        "#,
        url_list_html
    );

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
    .initialization_script(&inject_script)
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

    webview
        .window()
        .set_title(&format!("{}", url))
        .expect("Failed to set window title");

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
        .invoke_handler(tauri::generate_handler![
            greet,
            create_window,
            reset_timer,
            change_url,
            keyup
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

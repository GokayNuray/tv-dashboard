use tauri::window::Color;
use tauri::{command, AppHandle, Emitter, Manager, Url, WebviewUrl, WebviewWindowBuilder};

use log::info;

use std::fmt::Write;
use std::sync::Mutex;

#[command]
fn greet(name: &str) -> String {
    info!("Greet command invoked with name: {}", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}

static URLS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static CURRENT_URL: Mutex<String> = Mutex::new(String::new());
#[allow(dead_code)]
#[command]
fn create_list() -> String {
    let urls = URLS.lock().unwrap();
    info!("Creating list from URL list: {:?}", *urls);
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

    for url in &*urls {
        let safe_url = url.replace('\'', "\\'");
        let current_url = CURRENT_URL.lock().unwrap();
        let active = if *current_url == *url {"active-dot"} else {""};
        let _ = write!(
            url_list_html,
            r##"<div class="url-dot-container">
    <a href="#" class="url-dot {}" tabindex="0" onclick="window.__TAURI_INTERNALS__.invoke('change_url', {{ url: '{}' }}); return false;"></a>
    <span class="url-tooltip">{}</span>
</div>"##,
            active, safe_url, url
        );
    }
    url_list_html.push_str("</div>");
    url_list_html
}

#[command]
fn create_window(app_handle: AppHandle, urls: Vec<String>) {
    info!("Creating a new window...");

    let mut urls_mutex = URLS.lock().unwrap();
    *urls_mutex = urls;
    info!("URL list updated: {:?}", *urls_mutex);
    let webview_option = app_handle
        .get_webview("screen");
    if webview_option.is_some() {
        info!("Window with label 'screen' already exists, skipping creation.");
        webview_option.unwrap().reload().expect("Failed reload webview");
        return;
    };

    let inject_script = format!(
        r##"
        document.addEventListener('DOMContentLoaded', function() {{
        (async () => {{
            let list = await window.__TAURI_INTERNALS__.invoke('create_list');
            console.log('URL List:', list);
            document.body.insertAdjacentHTML('afterbegin', list);
            const dot = document.querySelector('.active-dot');
            console.log(dot);
            dot.style = 'border: 2px solid #00ff00';

        }})().catch((e) => console.error('Error creating URL list:', e));
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
        "##,
    );
    info!("{}", inject_script);

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

    let mut current_url = CURRENT_URL.lock().unwrap();
    *current_url = url.clone();

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
            keyup,
            create_list
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

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
static PAGE_CHANGE_TIMESTAMP: Mutex<i64> = Mutex::new(0);

#[allow(dead_code)]
#[command]
fn get_page_change_timestamp() -> i64 {
    *PAGE_CHANGE_TIMESTAMP.lock().unwrap()
}

#[allow(dead_code)]
#[command]
fn set_page_change_timestamp(timestamp: i64) {
    let mut page_change_timestamp = PAGE_CHANGE_TIMESTAMP.lock().unwrap();
    *page_change_timestamp = timestamp;
}

#[allow(dead_code)]
#[command]
fn create_list() -> String {
    let urls = URLS.lock().unwrap();
    info!("Creating list from URL list: {:?}", *urls);
    let mut url_list_html = String::from(
        r##"
        <style>
        #url-list .pie-container {
            width: 14px;
            height: 14px;
            position: absolute;
            top: 0;
            left: 0;
            display: flex;
            align-items: center;
            justify-content: center;
            font-family: Arial, sans-serif;
            font-size: 1em;
            font-weight: bold;
            pointer-events: none;
        }
        #url-list *,
        #url-list {
            position: initial;
            top: initial;
            left: initial;
            width: initial;
            height: initial;
            margin: initial;
            padding: initial;
            font-family: initial;
            font-size: initial;
            font-weight: initial;
            flex: initial;
            display: initial;
            align-items: initial;
            justify-content: initial;
            background: initial;
            color: initial;
            border: initial;
            box-shadow: initial;
            text-decoration: initial;
            cursor: initial;
            overflow: initial;
            z-index: initial;
            opacity: initial;
            visibility: initial;
            transition: initial;
            white-space: initial;
            pointer-events: initial;
            line-height: initial;
            border-radius: initial;
            flex-direction: initial;
            gap: initial;
            flex-wrap: initial;
            text-align: initial;
            box-sizing: initial;
            transform: initial;
        }
        #url-list {
            display: flex;
            flex-direction: column;
            gap: 8px;
            overflow: visible;
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
            flex-shrink: 0;
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
            word-break: break-all;
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
        <div id="url-list" style='position:fixed;top:0;left:0;padding:8px;z-index:9999;font-size:14px;max-width:300px;'>
    "##,
    );

    for url in &*urls {
        let safe_url = url.replace('\'', "\\'");
        let current_url = CURRENT_URL.lock().unwrap();
        let active = if *current_url == *url {"active-dot"} else {""};
        let _ = write!(
            url_list_html,
            r##"<div class="url-dot-container">
    <a href="#" class="url-dot {}" tabindex="0" onclick="window.__TAURI_INTERNALS__.invoke('change_url', {{ url: '{}', endTime: 0 }}); return false;">
        {}
    </a>
    <span class="url-tooltip">{}</span>
</div>"##,
            active,
            safe_url,
            if active == "active-dot" {
                // Insert the SVG pie for the active dot
                r##"<span class="pie-container">
                    <svg width="14" height="14">
                        <circle cx="7" cy="7" r="6" stroke="#21f623" stroke-width="1.5" fill="none"/>
                        <path id="pie-fill-active" fill="#21f623" stroke="none"/>
                    </svg>
                </span>"##
            } else {
                ""
            },
            url
        );
    }
    url_list_html.push_str("</div>");
    url_list_html
}

#[command]
fn create_window(app_handle: AppHandle, urls: Vec<String>) {
    info!("Creating a new window...");

    let mut urls_mutex = URLS.lock().unwrap();
    *urls_mutex = urls.clone();
    info!("URL list updated: {:?}", *urls_mutex);
    let webview_option = app_handle
        .get_webview("screen");
    if webview_option.is_some() {
        info!("Window with label 'screen' already exists, skipping creation.");
        let mut current_url = CURRENT_URL.lock().unwrap();
        *current_url = urls[0].clone();
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
            if (dot) {{
                const data = dot.getAttribute('data');
                dot.innerHTML = `
                    <span class="pie-container">
                        <svg width="14" height="14">
                            <circle cx="7" cy="7" r="6" stroke="#21f623" stroke-width="1.5" fill="none"/>
                            <path id="pie-fill-active" fill="#21f623" stroke="none"/>
                        </svg>
                    </span>
                `;
                function describeArc(cx, cy, r, percent) {{
                    const angle = percent / 100 * 360;
                    const radians = (angle - 90) * Math.PI / 180.0;
                    const x = cx + r * Math.cos(radians);
                    const y = cy + r * Math.sin(radians);
                    const largeArc = angle > 180 ? 1 : 0;
                    if (percent === 0) {{
                        return '';
                    }}
                    if (percent === 100) {{
                        return `M ${{cx}},${{cy}} m -${{r}},0 a ${{r}},${{r}} 0 1,0 ${{r*2}},0 a ${{r}},${{r}} 0 1,0 -${{r*2}},0`;
                    }}
                    return [
                        `M ${{cx}},${{cy}}`,
                        `L ${{cx}},${{cy - r}}`,
                        `A ${{r}},${{r}} 0 ${{largeArc}},1 ${{x}},${{y}}`,
                        'Z'
                    ].join(' ');
                }}
                function setProgress(percent) {{
                    const fillPath = document.getElementById('pie-fill-active');
                    if (fillPath) {{
                        fillPath.setAttribute('d', describeArc(7, 7, 6, percent));
                    }}
                }}
                let percent = 0;
                const target = 100;
                let startTime = Date.now();
                let endTime = await window.__TAURI_INTERNALS__.invoke('get_page_change_timestamp');
                const interval = setInterval(async () => {{
                    const newTime = await window.__TAURI_INTERNALS__.invoke('get_page_change_timestamp');
                    if (newTime !== endTime) {{
                        endTime = newTime;
                        startTime = Date.now();
                        return;
                    }}
                    const percent = Math.min(100, Math.floor(((Date.now() - startTime) / (endTime - startTime)) * 100));
                    setProgress(percent);
                }}, 100);
            }}
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
    .fullscreen(true)
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
fn change_url(app_handle: AppHandle, url: String, end_time: i64) {
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

    let mut page_open_timestamp = PAGE_CHANGE_TIMESTAMP.lock().unwrap();
    *page_open_timestamp = end_time;

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
            create_list,
            get_page_change_timestamp,
            set_page_change_timestamp
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    FileSystemDirectoryHandle, FileSystemFileHandle, FileSystemHandle, FileSystemHandleKind,
    FileSystemWritableFileStream,
};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        FileSystemBrowser {}

    }
}

#[derive(Debug, Clone, PartialEq)]
struct FileSystemEntry {
    name: String,
    is_directory: bool,
    children: Vec<FileSystemEntry>,
    handle: Option<FileSystemHandle>,
}

#[component]
pub fn FileSystemBrowser() -> Element {
    let mut selected_folder = use_signal(|| None::<String>);
    let mut file_structure = use_signal(Vec::<FileSystemEntry>::new);
    let mut last_update = use_signal(String::new);
    let selected_file = use_signal(|| None::<FileSystemHandle>);
    let mut file_content = use_signal(String::new);
    let mut is_content_modified = use_signal(|| false);

    let refresh_structure = move |handle: FileSystemDirectoryHandle| {
        spawn(async move {
            match read_directory_recursive(handle).await {
                Ok(entries) => {
                    file_structure.set(entries);
                    let now = js_sys::Date::new_0();
                    last_update.set(format!("Last updated: {}", now.to_string()));
                }
                Err(e) => {
                    web_sys::console::log_1(&format!("Error reading directory: {e:?}").into());
                }
            }
        });
    };

    let select_folder = move |_| {
        spawn(async move {
            match show_directory_picker().await {
                Ok(handle) => {
                    let name = handle.name();
                    selected_folder.set(Some(name));
                    refresh_structure(handle);
                }
                Err(e) => {
                    web_sys::console::log_1(&format!("Error selecting folder: {e:?}").into());
                }
            }
        });
    };

    rsx! {
        div {
            style: "padding: 20px; font-family: monospace; color: #ffffff; height: 100vh; display: flex; flex-direction: column;",
            h1 { style: "color: #ffffff; margin: 0 0 20px 0;", "File System Browser" }

            button {
                onclick: select_folder,
                style: "padding: 10px 20px; margin-bottom: 20px; font-size: 16px; background-color: #2a2a2a; color: #ffffff; border: 1px solid #444; border-radius: 5px; cursor: pointer;",
                "Select Folder"
            }

            if let Some(folder_name) = selected_folder() {
                div {
                    style: "margin-bottom: 10px; color: #ffffff;",
                    strong { "Selected Folder: " }
                    span { "{folder_name}" }
                }
            }

            div {
                style: "display: flex; flex: 1; gap: 20px; overflow: hidden;",

                // File tree panel
                div {
                    style: "flex: 0 0 40%; background: #1a1a1a; padding: 15px; border-radius: 5px; overflow-y: auto; white-space: pre; color: #ffffff; border: 1px solid #333; font-size: 14px; line-height: 1.6;",
                    for entry in file_structure() {
                        {render_file_tree(&entry, 0, selected_file, file_content, is_content_modified)}
                    }
                }

                // Editor panel
                if selected_file().is_some() {
                    div {
                        style: "flex: 1; display: flex; flex-direction: column; gap: 10px;",

                        textarea {
                            style: "flex: 1; background: #1a1a1a; color: #ffffff; border: 1px solid #333; border-radius: 5px; padding: 15px; font-family: monospace; font-size: 14px; resize: none;",
                            value: "{file_content()}",
                            oninput: move |evt| {
                                file_content.set(evt.value());
                                is_content_modified.set(true);
                            }
                        }

                        button {
                            style: "padding: 10px 20px; font-size: 16px; background-color: #2a7f2a; color: #ffffff; border: none; border-radius: 5px; cursor: pointer; align-self: flex-start;",
                            disabled: !is_content_modified(),
                            onclick: move |_| {
                                if let Some(handle) = selected_file() {
                                    let content = file_content();
                                    spawn(async move {
                                        let file_handle: FileSystemFileHandle = handle.unchecked_into();
                                        match write_file_content(&file_handle, &content).await {
                                            Ok(_) => {
                                                is_content_modified.set(false);
                                                web_sys::console::log_1(&"File saved successfully!".into());
                                            }
                                            Err(e) => {
                                                web_sys::console::log_1(&format!("Error writing file: {e:?}").into());
                                            }
                                        }
                                    });
                                }
                            },
                            "Apply Changes"
                        }
                    }
                }
            }
        }
    }
}

fn render_file_tree(
    entry: &FileSystemEntry,
    depth: usize,
    mut selected_file: Signal<Option<FileSystemHandle>>,
    mut file_content: Signal<String>,
    mut is_content_modified: Signal<bool>,
) -> Element {
    let indent = "  ".repeat(depth);
    let prefix = if entry.is_directory { "📁" } else { "📄" };

    // Check if this is a supported file type
    let is_supported_file = !entry.is_directory && {
        let name_lower = entry.name.to_lowercase();
        name_lower.ends_with(".json")
            || name_lower.ends_with(".md")
            || name_lower.ends_with(".rs")
            || name_lower.ends_with(".js")
    };

    let handle_clone = entry.handle.clone();
    let onclick = move |_| {
        if is_supported_file {
            if let Some(handle) = handle_clone.clone() {
                selected_file.set(Some(handle.clone()));
                is_content_modified.set(false);

                spawn(async move {
                    // Cast JsValue to our FileSystemFileHandle type
                    let file_handle: FileSystemFileHandle = handle.unchecked_into();
                    match read_file_content(&file_handle).await {
                        Ok(content) => {
                            file_content.set(content);
                        }
                        Err(e) => {
                            web_sys::console::log_1(&format!("Error reading file: {e:?}").into());
                        }
                    }
                });
            }
        }
    };

    rsx! {
        div {
            style: if is_supported_file { "cursor: pointer;" } else { "" },
            onclick: onclick,
            "{indent}{prefix} {entry.name}"
        }
        for child in &entry.children {
            {render_file_tree(child, depth + 1, selected_file, file_content, is_content_modified)}
        }
    }
}

async fn show_directory_picker() -> Result<FileSystemDirectoryHandle, JsValue> {
    let js_value = JsFuture::from(web_sys::window().unwrap().show_directory_picker()?).await?;

    js_value.dyn_into::<FileSystemDirectoryHandle>()
}

async fn read_file_content(handle: &FileSystemFileHandle) -> Result<String, JsValue> {
    // let file = handle.get_file().await?;
    let file = JsFuture::from(handle.get_file()).await?;
    let file: web_sys::File = file.dyn_into()?;
    let text_promise = file.text();
    let text = wasm_bindgen_futures::JsFuture::from(text_promise).await?;
    Ok(text.as_string().unwrap_or_default())
}

async fn write_file_content(handle: &FileSystemFileHandle, content: &str) -> Result<(), JsValue> {
    let writable = JsFuture::from(handle.create_writable()).await?;
    let stream: FileSystemWritableFileStream = writable.dyn_into()?;
    JsFuture::from(stream.write_with_str(content)?).await?;
    JsFuture::from(stream.close()).await?;
    Ok(())
}

fn read_directory_recursive(
    handle: FileSystemDirectoryHandle,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<FileSystemEntry>, JsValue>>>> {
    Box::pin(async move {
        let mut entries = Vec::new();

        // Use JavaScript interop for async iterator
        let values_method = js_sys::Reflect::get(&handle, &"values".into())
            .map_err(|_| JsValue::from_str("Failed to get values method"))?
            .dyn_into::<js_sys::Function>()
            .map_err(|_| JsValue::from_str("values is not a function"))?;

        let async_iterator = values_method
            .call0(&handle)
            .map_err(|_| JsValue::from_str("Failed to call values"))?;

        loop {
            let next_promise = js_sys::Reflect::get(&async_iterator, &"next".into())?
                .dyn_into::<js_sys::Function>()
                .map_err(|_| JsValue::from_str("Failed to get next function"))?
                .call0(&async_iterator)?;

            let result = wasm_bindgen_futures::JsFuture::from(
                next_promise
                    .dyn_into::<js_sys::Promise>()
                    .map_err(|_| JsValue::from_str("Failed to convert to promise"))?,
            )
            .await?;

            let done = js_sys::Reflect::get(&result, &"done".into())
                .map_err(|_| JsValue::from_str("Failed to get done"))?
                .as_bool()
                .unwrap_or(true);

            if done {
                break;
            }

            let value = js_sys::Reflect::get(&result, &"value".into())
                .map_err(|_| JsValue::from_str("Failed to get value"))?;

            let entry_handle = value
                .dyn_into::<FileSystemHandle>()
                .map_err(|_| JsValue::from_str("Failed to convert value to FileSystemHandle"))?;

            let name = entry_handle.name();
            let kind = entry_handle.kind();

            let is_directory = kind == FileSystemHandleKind::Directory;
            let mut children = Vec::new();

            if is_directory {
                if let Ok(dir_handle) = entry_handle.clone().dyn_into::<FileSystemDirectoryHandle>()
                {
                    children = read_directory_recursive(dir_handle)
                        .await
                        .unwrap_or_default();
                }
            }

            entries.push(FileSystemEntry {
                name,
                is_directory,
                children,
                handle: Some(entry_handle),
            });
        }

        entries.sort_by(|a, b| match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        Ok(entries)
    })
}

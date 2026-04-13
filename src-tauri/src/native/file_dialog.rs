// Author: Quadri Atharu
use std::path::PathBuf;

use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

pub fn open_file(
    app: &AppHandle,
    filters: Vec<(String, Vec<String>)>,
) -> Option<String> {
    let mut builder = app.dialog().file();
    for (name, extensions) in filters {
        let ext_refs: Vec<&str> = extensions.iter().map(|s| s.as_str()).collect();
        builder = builder.add_filter(&name, &ext_refs);
    }
    builder.blocking_file_path().map(|p| p.to_string_lossy().to_string())
}

pub fn save_file(
    app: &AppHandle,
    default_name: &str,
    filters: Vec<(String, Vec<String>)>,
) -> Option<String> {
    let mut builder = app.dialog().file();
    builder = builder.set_file_name(default_name);
    for (name, extensions) in filters {
        let ext_refs: Vec<&str> = extensions.iter().map(|s| s.as_str()).collect();
        builder = builder.add_filter(&name, &ext_refs);
    }
    builder.blocking_file_path().map(|p| p.to_string_lossy().to_string())
}

pub fn open_folder(_app: &AppHandle) -> Option<String> {
    if cfg!(target_os = "windows") {
        let script = "(New-Object -ComObject 'Shell.Application').BrowseForFolder(0, 'Select Folder', 0).Self.Path";
        let output = std::process::Command::new("powershell")
            .args(["-Command", script])
            .output()
            .ok()?;
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(path);
            }
        }
        None
    } else {
        let output = std::process::Command::new("zenity")
            .args(["--file-selection", "--directory"])
            .output()
            .ok()?;
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(path);
            }
        }
        None
    }
}

pub fn open_multiple_files(
    app: &AppHandle,
    filters: Vec<(String, Vec<String>)>,
) -> Option<Vec<String>> {
    let mut builder = app.dialog().file();
    for (name, extensions) in filters {
        let ext_refs: Vec<&str> = extensions.iter().map(|s| s.as_str()).collect();
        builder = builder.add_filter(&name, &ext_refs);
    }
    builder.blocking_file_paths().map(|paths| {
        paths.into_iter().map(|p| p.to_string_lossy().to_string()).collect()
    })
}

pub fn pick_image(app: &AppHandle) -> Option<PathBuf> {
    let builder = app
        .dialog()
        .file()
        .add_filter("Image", &["png", "jpg", "jpeg", "tiff", "tif", "bmp", "webp"]);
    builder.blocking_file_path()
}

pub fn pick_pdf(app: &AppHandle) -> Option<PathBuf> {
    let builder = app
        .dialog()
        .file()
        .add_filter("PDF Document", &["pdf"]);
    builder.blocking_file_path()
}

pub fn pick_spreadsheet(app: &AppHandle) -> Option<PathBuf> {
    let builder = app
        .dialog()
        .file()
        .add_filter("Spreadsheet", &["csv", "xlsx", "xls"]);
    builder.blocking_file_path()
}

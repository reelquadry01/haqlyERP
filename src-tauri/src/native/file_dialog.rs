use std::path::PathBuf;

use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

fn file_path_to_string(fp: tauri_plugin_dialog::FilePath) -> String {
    match fp {
        tauri_plugin_dialog::FilePath::Path(p) => p.to_string_lossy().to_string(),
        tauri_plugin_dialog::FilePath::Url(u) => u.to_string(),
    }
}

fn file_path_to_pathbuf(fp: tauri_plugin_dialog::FilePath) -> Option<PathBuf> {
    match fp {
        tauri_plugin_dialog::FilePath::Path(p) => Some(p),
        tauri_plugin_dialog::FilePath::Url(_) => None,
    }
}

pub fn open_file(
    app: &AppHandle,
    filters: Vec<(String, Vec<String>)>,
) -> Option<String> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut builder = app.dialog().file();
    for (name, extensions) in filters {
        let ext_refs: Vec<&str> = extensions.iter().map(|s| s.as_str()).collect();
        builder = builder.add_filter(&name, &ext_refs);
    }
    builder.pick_file(move |path| {
        let _ = tx.send(path.map(file_path_to_string));
    });
    rx.recv().ok().flatten()
}

pub fn save_file(
    app: &AppHandle,
    default_name: &str,
    filters: Vec<(String, Vec<String>)>,
) -> Option<String> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut builder = app.dialog().file();
    builder = builder.set_file_name(default_name);
    for (name, extensions) in filters {
        let ext_refs: Vec<&str> = extensions.iter().map(|s| s.as_str()).collect();
        builder = builder.add_filter(&name, &ext_refs);
    }
    builder.pick_file(move |path| {
        let _ = tx.send(path.map(file_path_to_string));
    });
    rx.recv().ok().flatten()
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
    let (tx, rx) = std::sync::mpsc::channel();
    let mut builder = app.dialog().file();
    for (name, extensions) in filters {
        let ext_refs: Vec<&str> = extensions.iter().map(|s| s.as_str()).collect();
        builder = builder.add_filter(&name, &ext_refs);
    }
    builder.pick_files(move |paths| {
        let _ = tx.send(paths.map(|ps| ps.into_iter().map(file_path_to_string).collect()));
    });
    rx.recv().ok().flatten()
}

pub fn pick_image(app: &AppHandle) -> Option<PathBuf> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .add_filter("Image", &["png", "jpg", "jpeg", "tiff", "tif", "bmp", "webp"])
        .pick_file(move |path| { let _ = tx.send(path.and_then(file_path_to_pathbuf)); });
    rx.recv().ok().flatten()
}

pub fn pick_pdf(app: &AppHandle) -> Option<PathBuf> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .add_filter("PDF Document", &["pdf"])
        .pick_file(move |path| { let _ = tx.send(path.and_then(file_path_to_pathbuf)); });
    rx.recv().ok().flatten()
}

pub fn pick_spreadsheet(app: &AppHandle) -> Option<PathBuf> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .add_filter("Spreadsheet", &["csv", "xlsx", "xls"])
        .pick_file(move |path| { let _ = tx.send(path.and_then(file_path_to_pathbuf)); });
    rx.recv().ok().flatten()
}

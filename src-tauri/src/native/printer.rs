// Author: Quadri Atharu
use std::path::Path;

use anyhow::{Context, Result};

pub fn print_file(file_path: &str) -> Result<()> {
    let path = Path::new(file_path);
    if !path.exists() {
        anyhow::bail!("File not found: {file_path}");
    }

    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match extension.as_str() {
        "pdf" => print_pdf(file_path),
        "txt" | "csv" | "json" | "html" | "htm" => print_text_file(file_path),
        "png" | "jpg" | "jpeg" | "bmp" | "tiff" | "tif" => print_image(file_path),
        _ => print_generic(file_path),
    }
}

fn print_pdf(file_path: &str) -> Result<()> {
    if cfg!(target_os = "windows") {
        print_windows(file_path)
    } else if cfg!(target_os = "macos") {
        let status = std::process::Command::new("lpr")
            .arg(file_path)
            .status()
            .context("Failed to execute lpr for PDF printing")?;
        if !status.success() {
            anyhow::bail!("lpr command failed for PDF: {file_path}");
        }
        Ok(())
    } else {
        let status = std::process::Command::new("lpr")
            .arg(file_path)
            .status()
            .context("Failed to execute lpr for PDF printing")?;
        if !status.success() {
            anyhow::bail!("lpr command failed for PDF: {file_path}");
        }
        Ok(())
    }
}

fn print_text_file(file_path: &str) -> Result<()> {
    if cfg!(target_os = "windows") {
        let status = std::process::Command::new("powershell")
            .args([
                "-Command",
                &format!("Start-Process -FilePath '{}' -Verb Print", file_path),
            ])
            .status()
            .context("Failed to execute PowerShell print command")?;
        if !status.success() {
            anyhow::bail!("Print command failed for text file: {file_path}");
        }
        Ok(())
    } else {
        let status = std::process::Command::new("lpr")
            .arg(file_path)
            .status()
            .context("Failed to execute lpr for text file")?;
        if !status.success() {
            anyhow::bail!("lpr command failed for text file: {file_path}");
        }
        Ok(())
    }
}

fn print_image(file_path: &str) -> Result<()> {
    if cfg!(target_os = "windows") {
        let status = std::process::Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Start-Process -FilePath '{}' -Verb Print",
                    file_path
                ),
            ])
            .status()
            .context("Failed to execute PowerShell print command for image")?;
        if !status.success() {
            anyhow::bail!("Print command failed for image: {file_path}");
        }
        Ok(())
    } else {
        let status = std::process::Command::new("lpr")
            .arg(file_path)
            .status()
            .context("Failed to execute lpr for image")?;
        if !status.success() {
            anyhow::bail!("lpr command failed for image: {file_path}");
        }
        Ok(())
    }
}

fn print_generic(file_path: &str) -> Result<()> {
    if cfg!(target_os = "windows") {
        print_windows(file_path)
    } else {
        let status = std::process::Command::new("lpr")
            .arg(file_path)
            .status()
            .context("Failed to execute lpr")?;
        if !status.success() {
            anyhow::bail!("lpr command failed: {file_path}");
        }
        Ok(())
    }
}

fn print_windows(file_path: &str) -> Result<()> {
    let status = std::process::Command::new("powershell")
        .args([
            "-Command",
            &format!("Start-Process -FilePath '{}' -Verb Print", file_path),
        ])
        .status()
        .context("Failed to execute PowerShell print command")?;

    if !status.success() {
        let fallback = std::process::Command::new("cmd")
            .args(["/c", "print", file_path])
            .status()
            .context("Fallback print command also failed")?;

        if !fallback.success() {
            anyhow::bail!("All print methods failed for: {file_path}");
        }
    }
    Ok(())
}

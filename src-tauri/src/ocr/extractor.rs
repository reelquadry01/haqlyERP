// Author: Quadri Atharu
use std::path::Path;

use anyhow::{Context, Result};

use crate::ocr::preprocessor::{DocumentFileType, ProcessedDocument};

pub fn extract(processed: &ProcessedDocument, language: &str) -> Result<String> {
    if !processed.needs_ocr {
        if !processed.text.is_empty() {
            return Ok(processed.text.clone());
        }
    }

    match processed.file_type {
        DocumentFileType::PdfNative => Ok(processed.text.clone()),
        DocumentFileType::Spreadsheet => Ok(processed.text.clone()),
        DocumentFileType::Image | DocumentFileType::PdfScanned => {
            extract_via_ocr(&processed.image_paths, language)
        }
    }
}

fn extract_via_ocr(image_paths: &[std::path::PathBuf], language: &str) -> Result<String> {
    if image_paths.is_empty() {
        anyhow::bail!("No images available for OCR; ensure pdftoppm is installed for scanned PDFs");
    }

    let mut all_text = String::new();

    for (i, image_path) in image_paths.iter().enumerate() {
        tracing::info!("Running Tesseract OCR on image {} of {}", i + 1, image_paths.len());

        let output = std::process::Command::new("tesseract")
            .arg(image_path)
            .arg("stdout")
            .arg("-l")
            .arg(language)
            .output()
            .with_context(|| {
                "Tesseract not found; install it and ensure it is on PATH"
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Tesseract failed for page {}: {stderr}", i + 1);
        }

        let page_text = String::from_utf8_lossy(&output.stdout);
        all_text.push_str(&page_text);
        if i < image_paths.len() - 1 {
            all_text.push_str("\n\n--- Page Break ---\n\n");
        }
    }

    tracing::info!(
        "OCR complete; extracted {} characters from {} pages",
        all_text.len(),
        image_paths.len()
    );

    Ok(all_text)
}

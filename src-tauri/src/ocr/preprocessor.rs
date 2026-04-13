// Author: Quadri Atharu
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentFileType {
    Image,
    PdfNative,
    PdfScanned,
    Spreadsheet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedDocument {
    pub file_type: DocumentFileType,
    pub text: String,
    pub pages: u32,
    pub needs_ocr: bool,
    pub image_paths: Vec<PathBuf>,
}

const MAX_DIMENSION: u32 = 4096;
const BINARIZATION_THRESHOLD: u8 = 128;
const MIN_TEXT_LENGTH_FOR_NATIVE_PDF: usize = 50;

pub fn preprocess(file_path: &Path, force_ocr: bool) -> Result<ProcessedDocument> {
    let extension = file_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match extension.as_str() {
        "pdf" => preprocess_pdf(file_path, force_ocr),
        "png" | "jpg" | "jpeg" | "tiff" | "tif" | "bmp" => preprocess_image(file_path),
        "csv" | "xlsx" | "xls" => preprocess_spreadsheet(file_path),
        _ => Err(anyhow::anyhow!(
            "Unsupported file format: .{extension}"
        )),
    }
}

fn preprocess_image(file_path: &Path) -> Result<ProcessedDocument> {
    let img = image::open(file_path)
        .with_context(|| format!("Failed to open image: {}", file_path.display()))?;

    let (width, height) = (img.width(), img.height());
    let mut resized = if width > MAX_DIMENSION || height > MAX_DIMENSION {
        tracing::info!(
            "Resizing image from {}x{} to fit within {}px",
            width,
            height,
            MAX_DIMENSION
        );
        img.resize(MAX_DIMENSION, MAX_DIMENSION, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };

    let gray = resized.to_luma8();

    let mut binary = gray;
    for pixel in binary.pixels_mut() {
        let val = pixel[0];
        *pixel = image::Luma([if val > BINARIZATION_THRESHOLD { 255u8 } else { 0u8 }]);
    }

    let temp_dir = std::env::temp_dir();
    let temp_name = format!("haqly_ocr_preprocessed_{}.png", uuid::Uuid::new_v4());
    let temp_path = temp_dir.join(&temp_name);

    image::DynamicImage::ImageLuma8(binary)
        .save(&temp_path)
        .with_context(|| "Failed to save preprocessed image")?;

    tracing::info!("Preprocessed image saved to: {}", temp_path.display());

    Ok(ProcessedDocument {
        file_type: DocumentFileType::Image,
        text: String::new(),
        pages: 1,
        needs_ocr: true,
        image_paths: vec![temp_path],
    })
}

fn preprocess_pdf(file_path: &Path, force_ocr: bool) -> Result<ProcessedDocument> {
    if force_ocr {
        tracing::info!("Force OCR requested; treating PDF as scanned");
        return preprocess_pdf_as_scanned(file_path);
    }

    let text = pdf_extract::extract_text(&file_path.to_string_lossy())
        .with_context(|| format!("Failed to extract text from PDF: {}", file_path.display()))?;

    if text.trim().len() >= MIN_TEXT_LENGTH_FOR_NATIVE_PDF {
        let page_count = estimate_pdf_pages(&text);
        tracing::info!(
            "PDF is native with {} characters of text (estimated {} pages)",
            text.len(),
            page_count
        );
        Ok(ProcessedDocument {
            file_type: DocumentFileType::PdfNative,
            text,
            pages: page_count,
            needs_ocr: false,
            image_paths: vec![],
        })
    } else {
        tracing::info!("PDF has minimal embedded text; treating as scanned document");
        preprocess_pdf_as_scanned(file_path)
    }
}

fn preprocess_pdf_as_scanned(file_path: &Path) -> Result<ProcessedDocument> {
    let image_paths = convert_pdf_to_images(file_path).unwrap_or_else(|e| {
        tracing::warn!("PDF-to-image conversion failed: {e}; OCR may be limited");
        vec![]
    });

    let pages = if image_paths.is_empty() {
        estimate_pdf_pages("")
    } else {
        image_paths.len() as u32
    };

    Ok(ProcessedDocument {
        file_type: DocumentFileType::PdfScanned,
        text: String::new(),
        pages,
        needs_ocr: true,
        image_paths,
    })
}

fn convert_pdf_to_images(pdf_path: &Path) -> Result<Vec<PathBuf>> {
    let output_dir = std::env::temp_dir().join(format!(
        "haqly_ocr_pdf_{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&output_dir)?;

    let output_prefix = output_dir.join("page");

    let status = std::process::Command::new("pdftoppm")
        .arg("-png")
        .arg("-r")
        .arg("300")
        .arg(pdf_path)
        .arg(&output_prefix)
        .status()
        .with_context(|| "pdftoppm not found; install poppler-utils")?;

    if !status.success() {
        anyhow::bail!("pdftoppm exited with non-zero status");
    }

    let mut images = Vec::new();
    for entry in std::fs::read_dir(&output_dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "png" {
                images.push(path);
            }
        }
    }
    images.sort();

    tracing::info!("Converted PDF to {} page images", images.len());
    Ok(images)
}

fn preprocess_spreadsheet(file_path: &Path) -> Result<ProcessedDocument> {
    let extension = file_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let text = if extension == "csv" {
        std::fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read CSV: {}", file_path.display()))?
    } else {
        tracing::warn!("XLSX/XLS parsing requires backend processing; sending path reference");
        format!("__SPREADSHEET_PATH__{}", file_path.display())
    };

    Ok(ProcessedDocument {
        file_type: DocumentFileType::Spreadsheet,
        text,
        pages: 1,
        needs_ocr: false,
        image_paths: vec![],
    })
}

fn estimate_pdf_pages(text: &str) -> u32 {
    if text.is_empty() {
        return 1;
    }
    let form_feed_count = text.matches('\x0c').count() as u32;
    if form_feed_count > 0 {
        form_feed_count + 1
    } else {
        let line_count = text.lines().count() as u32;
        (line_count / 50).max(1)
    }
}

# Author: Quadri Atharu
"""Document Intelligence module — OCR processing, classification, and lifecycle management."""

from .ocr_processor import OCRProcessor
from .document_classifier import DocumentClassifier
from .file_manager import FileManager

__all__ = [
    "OCRProcessor",
    "DocumentClassifier",
    "FileManager",
]

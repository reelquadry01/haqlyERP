# Author: Quadri Atharu
"""OCR Processor — extracts text via Tesseract and parses fields via Ollama LLM."""

import subprocess
import json
import logging
from typing import Optional
from pathlib import Path

logger = logging.getLogger(__name__)


class OCRProcessor:
    """Extracts text from document images using Tesseract and parses
    structured fields using an Ollama-hosted LLM."""

    def __init__(self, ollama_model: str = "llama3.2"):
        self.ollama_model = ollama_model

    def extract_text(self, file_path: str) -> str:
        try:
            result = subprocess.run(
                ["tesseract", file_path, "stdout", "--psm", "6"],
                capture_output=True,
                text=True,
                timeout=60,
            )
            if result.returncode != 0:
                logger.warning("Tesseract returned non-zero: %s", result.stderr.strip())
            return result.stdout.strip()
        except FileNotFoundError:
            logger.error("Tesseract not found on PATH")
            return ""
        except subprocess.TimeoutExpired:
            logger.error("Tesseract timed out for %s", file_path)
            return ""

    def _query_ollama(self, prompt: str) -> str:
        try:
            result = subprocess.run(
                ["ollama", "run", self.ollama_model, prompt],
                capture_output=True,
                text=True,
                timeout=120,
            )
            return result.stdout.strip()
        except FileNotFoundError:
            logger.error("Ollama not found on PATH")
            return "{}"
        except subprocess.TimeoutExpired:
            logger.error("Ollama timed out")
            return "{}"

    def extract_fields(
        self,
        file_path: str,
        document_type: str = "invoice",
    ) -> dict:
        raw_text = self.extract_text(file_path)
        if not raw_text:
            return {
                "raw_text": "",
                "fields": {},
                "document_type": document_type,
                "confidence": 0.0,
            }

        prompt = (
            f"You are a Nigerian finance document parser. Extract the following fields from the text below.\n"
            f"Document type: {document_type}\n"
            f"Fields to extract: vendor, amount, date, reference_number, suggested_account\n"
            f"Return ONLY a JSON object with those keys. Use null for missing values.\n\n"
            f"Text:\n{raw_text}\n"
        )

        llm_response = self._query_ollama(prompt)
        try:
            start = llm_response.index("{")
            end = llm_response.rindex("}") + 1
            fields = json.loads(llm_response[start:end])
        except (ValueError, json.JSONDecodeError):
            logger.warning("Failed to parse LLM response as JSON")
            fields = {
                "vendor": None,
                "amount": None,
                "date": None,
                "reference_number": None,
                "suggested_account": None,
            }

        return {
            "raw_text": raw_text,
            "fields": fields,
            "document_type": document_type,
            "confidence": 0.85 if any(v is not None for v in fields.values()) else 0.3,
        }

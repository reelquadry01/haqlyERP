# Author: Quadri Atharu
"""Document Classifier — classifies document type using Ollama LLM."""

import subprocess
import json
import logging
from typing import Optional

logger = logging.getLogger(__name__)

VALID_TYPES = ["invoice", "receipt", "bank_statement", "contract", "other"]


class DocumentClassifier:
    """Classifies a document into a known type based on its text content
    using an Ollama-hosted LLM."""

    def __init__(self, ollama_model: str = "llama3.2"):
        self.ollama_model = ollama_model

    def _query_ollama(self, prompt: str) -> str:
        try:
            result = subprocess.run(
                ["ollama", "run", self.ollama_model, prompt],
                capture_output=True,
                text=True,
                timeout=120,
            )
            return result.stdout.strip().lower()
        except FileNotFoundError:
            logger.error("Ollama not found on PATH")
            return "other"
        except subprocess.TimeoutExpired:
            logger.error("Ollama timed out")
            return "other"

    def classify(self, text_content: str) -> dict:
        if not text_content or not text_content.strip():
            return {
                "classification": "other",
                "confidence": 0.0,
                "alternatives": [],
            }

        prompt = (
            "You are a document classifier for Nigerian financial documents.\n"
            f"Classify the following document text into exactly one of: {', '.join(VALID_TYPES)}\n"
            "Return ONLY a JSON object with keys: classification, confidence (0-1), alternatives (list of other possible types with lower confidence).\n\n"
            f"Text:\n{text_content[:3000]}\n"
        )

        llm_response = self._query_ollama(prompt)
        try:
            start = llm_response.index("{")
            end = llm_response.rindex("}") + 1
            result = json.loads(llm_response[start:end])
            classification = result.get("classification", "other")
            if classification not in VALID_TYPES:
                classification = "other"
            return {
                "classification": classification,
                "confidence": min(float(result.get("confidence", 0.5)), 1.0),
                "alternatives": result.get("alternatives", []),
            }
        except (ValueError, json.JSONDecodeError, KeyError):
            for valid_type in VALID_TYPES:
                if valid_type in llm_response:
                    return {
                        "classification": valid_type,
                        "confidence": 0.6,
                        "alternatives": [],
                    }
            return {
                "classification": "other",
                "confidence": 0.3,
                "alternatives": [],
            }

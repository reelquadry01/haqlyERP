# Author: Quadri Atharu
"""File Manager — tracks document lifecycle (upload, processing, archive)."""

import json
import logging
from datetime import datetime
from pathlib import Path
from typing import Optional

logger = logging.getLogger(__name__)

STATUS_UPLOADED = "uploaded"
STATUS_PROCESSING = "processing"
STATUS_PROCESSED = "processed"
STATUS_ARCHIVED = "archived"

VALID_STATUSES = {STATUS_UPLOADED, STATUS_PROCESSING, STATUS_PROCESSED, STATUS_ARCHIVED}


class FileManager:
    """Manages the lifecycle of uploaded documents, tracking status
    transitions and metadata."""

    def __init__(self, tracker_dir: str = "./document_tracker"):
        self.tracker_dir = Path(tracker_dir)
        self.tracker_dir.mkdir(parents=True, exist_ok=True)

    def _tracker_path(self, doc_id: str) -> Path:
        return self.tracker_dir / f"{doc_id}.json"

    def _read_tracker(self, doc_id: str) -> Optional[dict]:
        path = self._tracker_path(doc_id)
        if not path.exists():
            return None
        try:
            with open(path, "r", encoding="utf-8") as f:
                return json.load(f)
        except (json.JSONDecodeError, OSError) as exc:
            logger.error("Failed to read tracker for %s: %s", doc_id, exc)
            return None

    def _write_tracker(self, doc_id: str, data: dict) -> None:
        path = self._tracker_path(doc_id)
        try:
            with open(path, "w", encoding="utf-8") as f:
                json.dump(data, f, indent=2, default=str)
        except OSError as exc:
            logger.error("Failed to write tracker for %s: %s", doc_id, exc)

    def track_upload(self, doc_id: str, metadata: dict) -> dict:
        record = {
            "doc_id": doc_id,
            "status": STATUS_UPLOADED,
            "metadata": metadata,
            "uploaded_at": datetime.utcnow().isoformat(),
            "processed_at": None,
            "archived_at": None,
        }
        self._write_tracker(doc_id, record)
        return record

    def mark_processed(self, doc_id: str) -> Optional[dict]:
        record = self._read_tracker(doc_id)
        if not record:
            logger.warning("Document %s not found in tracker", doc_id)
            return None
        if record["status"] not in (STATUS_UPLOADED, STATUS_PROCESSING):
            logger.warning(
                "Cannot mark %s as processed — current status: %s",
                doc_id,
                record["status"],
            )
            return record
        record["status"] = STATUS_PROCESSED
        record["processed_at"] = datetime.utcnow().isoformat()
        self._write_tracker(doc_id, record)
        return record

    def archive(self, doc_id: str) -> Optional[dict]:
        record = self._read_tracker(doc_id)
        if not record:
            logger.warning("Document %s not found in tracker", doc_id)
            return None
        record["status"] = STATUS_ARCHIVED
        record["archived_at"] = datetime.utcnow().isoformat()
        self._write_tracker(doc_id, record)
        return record

    def get_status(self, doc_id: str) -> Optional[dict]:
        record = self._read_tracker(doc_id)
        if not record:
            return None
        return {
            "doc_id": record["doc_id"],
            "status": record["status"],
            "uploaded_at": record.get("uploaded_at"),
            "processed_at": record.get("processed_at"),
            "archived_at": record.get("archived_at"),
        }

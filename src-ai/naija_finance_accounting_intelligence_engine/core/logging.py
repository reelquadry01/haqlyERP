# Author: Quadri Atharu
"""Structured logging setup with JSON formatter."""

from __future__ import annotations

import json
import logging
import sys
from datetime import datetime, timezone
from typing import Any, Dict, Optional


class JsonFormatter(logging.Formatter):
    """Formats log records as single-line JSON objects for structured logging."""

    def format(self, record: logging.LogRecord) -> str:
        log_entry: Dict[str, Any] = {
            "timestamp": datetime.now(tz=timezone.utc).isoformat(),
            "level": record.levelname,
            "logger": record.name,
            "message": record.getMessage(),
        }
        if record.exc_info and record.exc_info[1] is not None:
            log_entry["exception"] = self.formatException(record.exc_info)
        for key in ("module", "funcName", "lineno"):
            if hasattr(record, key):
                log_entry[key] = getattr(record, key)
        extra = getattr(record, "extra_fields", None)
        if isinstance(extra, dict):
            log_entry.update(extra)
        return json.dumps(log_entry, default=str, ensure_ascii=False)


class _StructLoggerAdapter(logging.LoggerAdapter):
    """Logger adapter that packs extra keyword arguments into a single extra_fields dict."""

    def log(self, level: int, msg: object, /, *args: Any, **kwargs: Any) -> None:
        extra_fields = kwargs.pop("extra_fields", None)
        if extra_fields is None:
            collected = {k: v for k, v in kwargs.items() if not k.startswith("_")}
            if collected:
                extra_fields = collected
        kwargs["extra"] = {**kwargs.get("extra", {}), "extra_fields": extra_fields or {}}
        super().log(level, msg, *args, **kwargs)

    def info(self, msg: object, /, *args: Any, **kwargs: Any) -> None:
        self.log(logging.INFO, msg, *args, **kwargs)

    def warning(self, msg: object, /, *args: Any, **kwargs: Any) -> None:
        self.log(logging.WARNING, msg, *args, **kwargs)

    def error(self, msg: object, /, *args: Any, **kwargs: Any) -> None:
        self.log(logging.ERROR, msg, *args, **kwargs)

    def debug(self, msg: object, /, *args: Any, **kwargs: Any) -> None:
        self.log(logging.DEBUG, msg, *args, **kwargs)

    def critical(self, msg: object, /, *args: Any, **kwargs: Any) -> None:
        self.log(logging.CRITICAL, msg, *args, **kwargs)


_handler: Optional[logging.StreamHandler] = None


def _ensure_handler() -> logging.StreamHandler:
    global _handler
    if _handler is None:
        _handler = logging.StreamHandler(sys.stdout)
        _handler.setFormatter(JsonFormatter())
    return _handler


def get_logger(name: str, level: Optional[str] = None) -> _StructLoggerAdapter:
    """Return a structured JSON logger for the given module name."""
    from .config import settings

    logger = logging.getLogger(name)
    logger.setLevel(getattr(logging, (level or settings.log_level).upper(), logging.INFO))
    if not logger.handlers:
        logger.addHandler(_ensure_handler())
    return _StructLoggerAdapter(logger, extra={"extra_fields": {}})

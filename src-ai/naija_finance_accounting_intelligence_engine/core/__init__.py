# Author: Quadri Atharu
"""Core module — config, engine, logging, exceptions, registry."""

from .config import Settings, settings
from .engine import FinanceIntelligenceEngine
from .exceptions import (
    AccountingError,
    AnalysisError,
    EngineError,
    IFRSError,
    OcrError,
    RiskError,
    TaxError,
    ValidationError,
)
from .logging import get_logger
from .registry import ModuleRegistry

__all__ = [
    "Settings",
    "settings",
    "FinanceIntelligenceEngine",
    "AccountingError",
    "AnalysisError",
    "EngineError",
    "IFRSError",
    "OcrError",
    "RiskError",
    "TaxError",
    "ValidationError",
    "get_logger",
    "ModuleRegistry",
]

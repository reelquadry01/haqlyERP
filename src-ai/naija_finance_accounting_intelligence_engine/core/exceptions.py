# Author: Quadri Atharu
"""Custom exceptions for the Nigerian Finance & Accounting Intelligence Engine."""


class EngineError(Exception):
    """Base exception for all engine errors."""

    def __init__(self, message: str = "", code: str = "ENGINE_ERROR", details: dict | None = None) -> None:
        self.message = message
        self.code = code
        self.details = details or {}
        super().__init__(message)


class AccountingError(EngineError):
    """Raised when an accounting operation fails."""

    def __init__(self, message: str = "Accounting operation failed", details: dict | None = None) -> None:
        super().__init__(message=message, code="ACCOUNTING_ERROR", details=details)


class TaxError(EngineError):
    """Raised when a tax computation or filing operation fails."""

    def __init__(self, message: str = "Tax computation failed", details: dict | None = None) -> None:
        super().__init__(message=message, code="TAX_ERROR", details=details)


class ValidationError(EngineError):
    """Raised when data validation fails."""

    def __init__(self, message: str = "Validation failed", field: str = "", details: dict | None = None) -> None:
        self.field = field
        super().__init__(message=message, code="VALIDATION_ERROR", details=details)


class IFRSError(EngineError):
    """Raised when an IFRS compliance issue is detected."""

    def __init__(self, message: str = "IFRS compliance error", standard: str = "", details: dict | None = None) -> None:
        self.standard = standard
        super().__init__(message=message, code="IFRS_ERROR", details=details)


class AnalysisError(EngineError):
    """Raised when a financial analysis operation fails."""

    def __init__(self, message: str = "Financial analysis failed", details: dict | None = None) -> None:
        super().__init__(message=message, code="ANALYSIS_ERROR", details=details)


class RiskError(EngineError):
    """Raised when a risk assessment operation fails."""

    def __init__(self, message: str = "Risk assessment failed", details: dict | None = None) -> None:
        super().__init__(message=message, code="RISK_ERROR", details=details)


class OcrError(EngineError):
    """Raised when an OCR/document processing operation fails."""

    def __init__(self, message: str = "OCR processing failed", details: dict | None = None) -> None:
        super().__init__(message=message, code="OCR_ERROR", details=details)

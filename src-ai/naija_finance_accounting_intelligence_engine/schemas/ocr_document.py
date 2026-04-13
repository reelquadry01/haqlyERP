# Author: Quadri Atharu
"""OCR Document Pydantic v2 models."""

from __future__ import annotations

from datetime import datetime
from enum import Enum
from typing import Dict, List, Optional

from pydantic import BaseModel, Field, ConfigDict


class DocumentClassification(str, Enum):
    INVOICE = "INVOICE"
    RECEIPT = "RECEIPT"
    PURCHASE_ORDER = "PURCHASE_ORDER"
    BANK_STATEMENT = "BANK_STATEMENT"
    TAX_ASSESSMENT = "TAX_ASSESSMENT"
    CONTRACT = "CONTRACT"
    WAYBILL = "WAYBILL"
    DELIVERY_NOTE = "DELIVERY_NOTE"
    CREDIT_NOTE = "CREDIT_NOTE"
    DEBIT_NOTE = "DEBIT_NOTE"
    PAYSLIP = "PAYSLIP"
    UNKNOWN = "UNKNOWN"


class ExtractedField(BaseModel):
    field_name: str = Field(..., min_length=1, max_length=100)
    value: str = Field(default="", max_length=2000)
    confidence: float = Field(default=0.0, ge=0.0, le=1.0)
    bounding_box: Optional[Dict[str, float]] = Field(default=None, description="x, y, width, height")
    page_number: Optional[int] = Field(default=None, ge=1)


class LineItemExtraction(BaseModel):
    description: str = Field(default="", max_length=500)
    quantity: float = Field(default=0.0, ge=0)
    unit_price: float = Field(default=0.0, ge=0)
    line_total: float = Field(default=0.0, ge=0)
    tax_amount: float = Field(default=0.0, ge=0)
    account_code_suggestion: Optional[str] = Field(default=None, pattern=r"^\d{4,5}$")
    confidence: float = Field(default=0.0, ge=0.0, le=1.0)


class AccountSuggestion(BaseModel):
    account_code: str = Field(..., pattern=r"^\d{4,5}$")
    account_name: str
    confidence: float = Field(default=0.0, ge=0.0, le=1.0)
    reason: str = Field(default="", max_length=500)


class OcrResult(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: str
    document_path: str
    document_type: DocumentClassification = DocumentClassification.UNKNOWN
    classification_confidence: float = Field(default=0.0, ge=0.0, le=1.0)
    extracted_fields: List[ExtractedField] = Field(default_factory=list)
    line_items: List[LineItemExtraction] = Field(default_factory=list)
    account_suggestions: List[AccountSuggestion] = Field(default_factory=list)
    total_amount: Optional[float] = Field(default=None, ge=0)
    vendor_name: Optional[str] = None
    document_date: Optional[datetime] = None
    document_number: Optional[str] = None
    processing_time_ms: float = 0.0
    ocr_engine: str = Field(default="ollama", max_length=50)
    model_used: Optional[str] = None
    raw_text: Optional[str] = None
    processed_at: datetime = Field(default_factory=datetime.now)

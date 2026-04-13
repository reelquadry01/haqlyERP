# Author: Quadri Atharu
"""Tax Pydantic v2 models."""

from __future__ import annotations

from datetime import datetime
from enum import Enum
from typing import Any, Dict, List, Optional

from pydantic import BaseModel, Field, ConfigDict


class TaxType(str, Enum):
    VAT = "VAT"
    WHT = "WHT"
    CIT = "CIT"
    EDU_TAX = "EDU_TAX"
    CGT = "CGT"
    STAMP_DUTY = "STAMP_DUTY"
    PAYE = "PAYE"


class TaxComputationRequest(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    company_id: str
    tax_type: TaxType
    period_start: datetime
    period_end: datetime
    taxable_amount: float = Field(default=0.0, ge=0)
    turnover: float = Field(default=0.0, ge=0)
    profit_before_tax: float = Field(default=0.0)
    industry: str = Field(default="general")
    wht_category: Optional[str] = Field(default=None, description="WHT payment category")
    disposal_proceeds: Optional[float] = Field(default=None, ge=0)
    cost_basis: Optional[float] = Field(default=None, ge=0)
    allowable_deductions: Optional[float] = Field(default=None, ge=0)
    document_type: Optional[str] = Field(default=None, description="For stamp duties")
    document_value: Optional[float] = Field(default=None, ge=0)
    assessable_profit: Optional[float] = Field(default=None, ge=0)
    extra_params: Optional[Dict[str, Any]] = Field(default=None)


class TaxComputationResult(BaseModel):
    tax_type: TaxType
    tax_amount: float = 0.0
    taxable_base: float = 0.0
    rate_applied: float = 0.0
    exemptions: float = 0.0
    details: Dict[str, Any] = Field(default_factory=dict)
    warnings: List[str] = Field(default_factory=list)
    computed_at: datetime = Field(default_factory=datetime.now)


class TaxSchedule(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: str
    company_id: str
    tax_type: TaxType
    period_start: datetime
    period_end: datetime
    line_items: List[Dict[str, Any]] = Field(default_factory=list)
    total_tax: float = 0.0
    total_taxable: float = 0.0
    generated_at: datetime = Field(default_factory=datetime.now)


class TaxReturn(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: str
    company_id: str
    tax_type: TaxType
    period_start: datetime
    period_end: datetime
    form_data: Dict[str, Any] = Field(default_factory=dict)
    tax_due: float = 0.0
    filing_deadline: Optional[datetime] = None
    status: str = "draft"
    submitted_at: Optional[datetime] = None
    created_at: datetime = Field(default_factory=datetime.now)


class TaxRiskFlag(BaseModel):
    risk_type: str
    severity: str = Field(default="medium", pattern=r"^(low|medium|high|critical)$")
    tax_type: TaxType
    description: str
    metric_value: Optional[float] = None
    threshold_value: Optional[float] = None
    deviation_pct: Optional[float] = None
    recommendation: Optional[str] = None
    detected_at: datetime = Field(default_factory=datetime.now)

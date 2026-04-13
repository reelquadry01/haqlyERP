# Author: Quadri Atharu
"""Audit Pydantic v2 models."""

from __future__ import annotations

from datetime import datetime
from enum import Enum
from typing import Any, Dict, List, Optional

from pydantic import BaseModel, Field, ConfigDict


class AuditEntry(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: str
    company_id: str
    user_id: Optional[str] = None
    action: str = Field(..., description="E.g. CREATE, UPDATE, DELETE, POST, APPROVE, REVERSE")
    entity_type: str = Field(..., description="E.g. journal_entry, invoice, tax_return")
    entity_id: str
    old_value: Optional[Dict[str, Any]] = None
    new_value: Optional[Dict[str, Any]] = None
    timestamp: datetime = Field(default_factory=datetime.now)
    ip_address: Optional[str] = None
    user_agent: Optional[str] = None
    notes: Optional[str] = None


class SamplingRequest(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    company_id: str
    population: List[Dict[str, Any]] = Field(..., min_length=1)
    sample_size: int = Field(default=25, ge=1)
    method: str = Field(default="random", pattern=r"^(random|systematic|stratified|monetary_unit|sequential)$")
    stratification_key: Optional[str] = Field(default=None, description="Field to stratify on")
    materiality_threshold: float = Field(default=1_000_000, ge=0)
    confidence_level: float = Field(default=0.95, ge=0.80, le=0.99)


class ExceptionFlag(BaseModel):
    flag_type: str = Field(..., description="E.g. unusual_amount, duplicate, missing_doc, late_entry")
    severity: str = Field(default="medium", pattern=r"^(low|medium|high|critical)$")
    entity_type: str
    entity_id: str
    description: str
    metric_value: Optional[float] = None
    threshold: Optional[float] = None
    recommendation: Optional[str] = None
    detected_at: datetime = Field(default_factory=datetime.now)
    reviewed: bool = Field(default=False)
    reviewed_by: Optional[str] = None
    reviewed_at: Optional[datetime] = None


class AuditReport(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: str
    company_id: str
    period_start: datetime
    period_end: datetime
    total_entries_audited: int = 0
    exceptions_found: List[ExceptionFlag] = Field(default_factory=list)
    sampling_method: str = "random"
    sample_size: int = 0
    materiality: float = 0.0
    risk_assessment: Optional[Dict[str, Any]] = None
    generated_at: datetime = Field(default_factory=datetime.now)
    generated_by: Optional[str] = None

# Author: Quadri Atharu
"""Budget Pydantic v2 models."""

from __future__ import annotations

from datetime import datetime
from typing import Optional

from pydantic import BaseModel, Field, ConfigDict


class BudgetLine(BaseModel):
    account_code: str = Field(..., pattern=r"^\d{4,5}$")
    account_name: str
    period: str = Field(..., description="Budget period e.g. 2026-Q1 or 2026-01")
    budgeted_amount: float = Field(..., ge=0)
    notes: Optional[str] = None


class BudgetCreate(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    company_id: str
    fiscal_year: int = Field(..., ge=2000, le=2100)
    name: str = Field(..., min_length=1, max_length=200)
    description: Optional[str] = None
    lines: list[BudgetLine] = Field(..., min_length=1)
    approved_by: Optional[str] = None
    status: str = Field(default="draft", pattern=r"^(draft|submitted|approved|locked)$")


class BudgetActual(BaseModel):
    account_code: str
    period: str
    budgeted_amount: float = 0.0
    actual_amount: float = 0.0


class VarianceReport(BaseModel):
    company_id: str
    fiscal_year: int
    budget_name: str
    period: str
    lines: list[BudgetActual] = Field(default_factory=list)
    total_budgeted: float = 0.0
    total_actual: float = 0.0
    total_variance: float = 0.0
    variance_pct: float = 0.0
    generated_at: datetime = Field(default_factory=datetime.now)

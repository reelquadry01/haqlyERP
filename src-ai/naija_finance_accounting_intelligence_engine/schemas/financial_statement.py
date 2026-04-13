# Author: Quadri Atharu
"""Financial statement Pydantic v2 models."""

from __future__ import annotations

from datetime import datetime
from typing import Dict, List, Optional

from pydantic import BaseModel, Field, ConfigDict


class StatementLine(BaseModel):
    label: str = Field(..., min_length=1, max_length=200)
    amount: float = 0.0
    sub_lines: List["StatementLine"] = Field(default_factory=list)
    indent_level: int = Field(default=0, ge=0)
    is_subtotal: bool = Field(default=False)
    note_reference: Optional[str] = None


class FinancialStatementRequest(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    company_id: str
    period_start: datetime
    period_end: datetime
    currency: str = Field(default="NGN", max_length=3)
    comparative: bool = Field(default=False, description="Include comparative prior period")
    include_notes: bool = Field(default=True)
    consolidation: bool = Field(default=False, description="Consolidated group statement")


class IncomeStatement(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    company_id: str
    period_start: datetime
    period_end: datetime
    currency: str = "NGN"
    revenue: List[StatementLine] = Field(default_factory=list)
    cost_of_sales: List[StatementLine] = Field(default_factory=list)
    gross_profit: float = 0.0
    operating_expenses: List[StatementLine] = Field(default_factory=list)
    operating_income: float = 0.0
    finance_costs: List[StatementLine] = Field(default_factory=list)
    other_income: List[StatementLine] = Field(default_factory=list)
    profit_before_tax: float = 0.0
    tax_expense: float = 0.0
    net_income: float = 0.0
    comprehensive_income: Optional[float] = None
    eps: Optional[float] = None
    comparative: Optional["IncomeStatement"] = None


class BalanceSheet(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    company_id: str
    as_of: datetime
    currency: str = "NGN"
    non_current_assets: List[StatementLine] = Field(default_factory=list)
    current_assets: List[StatementLine] = Field(default_factory=list)
    total_assets: float = 0.0
    equity: List[StatementLine] = Field(default_factory=list)
    non_current_liabilities: List[StatementLine] = Field(default_factory=list)
    current_liabilities: List[StatementLine] = Field(default_factory=list)
    total_liabilities: float = 0.0
    total_equity: float = 0.0
    balance_check: bool = False
    comparative: Optional["BalanceSheet"] = None


class CashFlowStatement(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    company_id: str
    period_start: datetime
    period_end: datetime
    currency: str = "NGN"
    operating_activities: List[StatementLine] = Field(default_factory=list)
    net_cash_from_operating: float = 0.0
    investing_activities: List[StatementLine] = Field(default_factory=list)
    net_cash_from_investing: float = 0.0
    financing_activities: List[StatementLine] = Field(default_factory=list)
    net_cash_from_financing: float = 0.0
    net_change_in_cash: float = 0.0
    cash_at_beginning: float = 0.0
    cash_at_end: float = 0.0
    comparative: Optional["CashFlowStatement"] = None

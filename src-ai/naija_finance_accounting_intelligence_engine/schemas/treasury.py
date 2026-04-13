# Author: Quadri Atharu
"""Treasury Pydantic v2 models."""

from __future__ import annotations

from datetime import datetime
from typing import Dict, List, Optional

from pydantic import BaseModel, Field, ConfigDict


class CashPosition(BaseModel):
    company_id: str
    as_of: datetime
    currency: str = Field(default="NGN", max_length=3)
    bank_balances: Dict[str, float] = Field(default_factory=dict, description="Bank name -> balance")
    petty_cash: float = Field(default=0.0, ge=0)
    undeposited_funds: float = Field(default=0.0, ge=0)
    total_available: float = Field(default=0.0)
    outstanding_cheques: float = Field(default=0.0, ge=0)
    outstanding_deposits: float = Field(default=0.0, ge=0)
    restricted_funds: float = Field(default=0.0, ge=0, description="Escrow or restricted cash")


class BankReconciliation(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    company_id: str
    bank_name: str
    account_number: str = Field(..., max_length=30)
    period_end: datetime
    currency: str = Field(default="NGN", max_length=3)
    book_balance: float = 0.0
    bank_balance: float = 0.0
    outstanding_deposits: List[Dict[str, object]] = Field(default_factory=list)
    outstanding_cheques: List[Dict[str, object]] = Field(default_factory=list)
    bank_charges: float = 0.0
    bank_interest: float = 0.0
    errors: List[Dict[str, object]] = Field(default_factory=list)
    adjusted_book_balance: float = 0.0
    adjusted_bank_balance: float = 0.0
    difference: float = 0.0
    reconciled: bool = False
    reconciled_by: Optional[str] = None
    reconciled_at: Optional[datetime] = None


class InterestEntry(BaseModel):
    period: int = Field(..., ge=1, description="Period number")
    opening_balance: float = 0.0
    interest_charge: float = 0.0
    principal_repayment: float = 0.0
    closing_balance: float = 0.0
    payment_date: Optional[datetime] = None


class LoanSchedule(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    company_id: str
    loan_id: Optional[str] = None
    loan_name: str = Field(..., min_length=1, max_length=200)
    principal: float = Field(..., gt=0)
    annual_rate: float = Field(..., gt=0, le=1.0, description="Annual interest rate as decimal")
    term_months: int = Field(..., ge=1)
    start_date: datetime
    payment_frequency: str = Field(default="monthly", pattern=r"^(monthly|quarterly|annually)$")
    repayment_type: str = Field(default="reducing_balance", pattern=r"^(reducing_balance|flat|interest_only)$")
    currency: str = Field(default="NGN", max_length=3)
    emi: float = 0.0
    total_payment: float = 0.0
    total_interest: float = 0.0
    schedule: List[InterestEntry] = Field(default_factory=list)

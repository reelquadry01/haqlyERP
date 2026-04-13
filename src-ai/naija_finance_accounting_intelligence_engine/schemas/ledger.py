# Author: Quadri Atharu
"""Ledger Pydantic v2 models."""

from __future__ import annotations

from datetime import datetime
from typing import Optional

from pydantic import BaseModel, Field, ConfigDict


class LedgerEntry(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: str
    account_code: str = Field(..., pattern=r"^\d{4,5}$")
    account_name: str
    journal_entry_id: str
    entry_date: datetime
    description: str
    debit: float = 0.0
    credit: float = 0.0
    balance: float = 0.0
    cost_center: Optional[str] = None
    reference: Optional[str] = None
    created_at: datetime


class LedgerBalance(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    account_code: str = Field(..., pattern=r"^\d{4,5}$")
    account_name: str
    account_type: str
    opening_balance: float = 0.0
    total_debit: float = 0.0
    total_credit: float = 0.0
    closing_balance: float = 0.0
    period_start: datetime
    period_end: datetime
    entry_count: int = 0


class SubLedgerEntry(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: str
    subledger_type: str = Field(..., description="E.g. accounts_receivable, accounts_payable, fixed_assets")
    control_account_code: str = Field(..., pattern=r"^\d{4,5}$")
    subledger_account_id: str
    journal_entry_id: str
    entry_date: datetime
    description: str
    debit: float = 0.0
    credit: float = 0.0
    balance: float = 0.0
    counterparty: Optional[str] = None
    reference: Optional[str] = None
    due_date: Optional[datetime] = None
    created_at: datetime

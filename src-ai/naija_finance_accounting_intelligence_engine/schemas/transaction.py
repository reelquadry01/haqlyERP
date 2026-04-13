# Author: Quadri Atharu
"""Transaction Pydantic v2 models."""

from __future__ import annotations

from datetime import datetime
from enum import Enum
from typing import Optional

from pydantic import BaseModel, Field, ConfigDict


class TransactionType(str, Enum):
    SALES = "SALES"
    PURCHASE = "PURCHASE"
    CASH_RECEIPT = "CASH_RECEIPT"
    CASH_PAYMENT = "CASH_PAYMENT"
    JOURNAL = "JOURNAL"
    DEBIT_NOTE = "DEBIT_NOTE"
    CREDIT_NOTE = "CREDIT_NOTE"
    INVOICE = "INVOICE"
    BILL = "BILL"
    EXPENSE = "EXPENSE"
    TRANSFER = "TRANSFER"
    SALARY = "SALARY"
    TAX_PAYMENT = "TAX_PAYMENT"
    DEPRECIATION = "DEPRECIATION"
    ADJUSTMENT = "ADJUSTMENT"


class AccountingMethod(str, Enum):
    ACCRUAL = "ACCRUAL"
    CASH = "CASH"
    MODIFIED_CASH = "MODIFIED_CASH"


class TransactionBase(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    transaction_type: TransactionType
    accounting_method: AccountingMethod = AccountingMethod.ACCRUAL
    amount: float = Field(..., gt=0, description="Transaction amount in NGN")
    description: str = Field(..., min_length=1, max_length=500)
    transaction_date: datetime
    reference: Optional[str] = Field(default=None, max_length=100)
    counterparty: Optional[str] = Field(default=None, max_length=200)
    department: Optional[str] = Field(default=None, max_length=100)
    project: Optional[str] = Field(default=None, max_length=100)
    currency: str = Field(default="NGN", max_length=3)
    exchange_rate: float = Field(default=1.0, gt=0)
    is_taxable: bool = Field(default=True)
    tax_inclusive: bool = Field(default=False, description="Whether the amount includes VAT")


class TransactionCreate(TransactionBase):
    company_id: str
    debit_account_code: str = Field(..., pattern=r"^\d{4,5}$")
    credit_account_code: str = Field(..., pattern=r"^\d{4,5}$")
    tax_lines: Optional[list[dict]] = Field(default=None, description="Tax breakdown lines")
    attachments: Optional[list[str]] = Field(default=None, description="Document attachment paths")
    metadata: Optional[dict] = Field(default=None, description="Additional metadata")


class TransactionResponse(TransactionBase):
    model_config = ConfigDict(from_attributes=True)

    id: str
    company_id: str
    debit_account_code: str
    credit_account_code: str
    status: str = "pending"
    journal_entry_id: Optional[str] = None
    created_at: datetime
    updated_at: datetime

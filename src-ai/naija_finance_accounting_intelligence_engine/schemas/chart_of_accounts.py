# Author: Quadri Atharu
"""Chart of Accounts Pydantic v2 models."""

from __future__ import annotations

from datetime import datetime
from enum import Enum
from typing import Optional

from pydantic import BaseModel, Field, ConfigDict


class AccountType(str, Enum):
    ASSET = "ASSET"
    LIABILITY = "LIABILITY"
    EQUITY = "EQUITY"
    REVENUE = "REVENUE"
    EXPENSE = "EXPENSE"


class AccountBase(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    code: str = Field(..., pattern=r"^\d{4,5}$", description="Account code (4-5 digits)")
    name: str = Field(..., min_length=1, max_length=200, description="Account name")
    account_type: AccountType
    parent_code: Optional[str] = Field(default=None, pattern=r"^\d{4,5}$", description="Parent account code for hierarchy")
    is_control_account: bool = Field(default=False, description="Whether this is a control/sub-ledger account")
    is_active: bool = Field(default=True)
    description: Optional[str] = Field(default=None, max_length=500)
    tax_related: bool = Field(default=False, description="Whether this account is used in tax computations")
    industry_tag: Optional[str] = Field(default=None, description="Industry tag for template matching")


class AccountCreate(AccountBase):
    company_id: str = Field(..., description="Company ID this account belongs to")
    opening_balance: float = Field(default=0.0, ge=0)


class AccountResponse(AccountBase):
    model_config = ConfigDict(from_attributes=True)

    id: str
    company_id: str
    current_balance: float = 0.0
    opening_balance: float = 0.0
    created_at: datetime
    updated_at: datetime

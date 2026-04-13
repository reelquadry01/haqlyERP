# Author: Quadri Atharu
"""Company and consolidation Pydantic v2 models."""

from __future__ import annotations

from datetime import datetime
from enum import Enum
from typing import List, Optional

from pydantic import BaseModel, Field, ConfigDict


class CompanyBase(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    name: str = Field(..., min_length=1, max_length=300)
    registration_number: str = Field(..., min_length=1, max_length=50, description="CAC registration number")
    tax_identification_number: str = Field(..., min_length=1, max_length=20, description="TIN")
    industry: str = Field(default="general", max_length=100)
    address: Optional[str] = Field(default=None, max_length=500)
    city: Optional[str] = Field(default=None, max_length=100)
    state: Optional[str] = Field(default=None, max_length=100)
    country: str = Field(default="Nigeria")
    currency: str = Field(default="NGN", max_length=3)
    fiscal_year_start_month: int = Field(default=1, ge=1, le=12)
    accounting_method: str = Field(default="accrual", pattern=r"^(accrual|cash|modified_cash)$")
    is_group: bool = Field(default=False, description="Whether this is a parent/group company")
    parent_company_id: Optional[str] = Field(default=None, description="Parent company ID for subsidiaries")
    vat_registration_number: Optional[str] = Field(default=None, max_length=30)
    is_active: bool = Field(default=True)


class ConsolidationRequest(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    parent_company_id: str
    period_start: datetime
    period_end: datetime
    subsidiary_ids: List[str] = Field(..., min_length=1)
    currency: str = Field(default="NGN", max_length=3)
    elimination_entries: List["IntercompanyTransaction"] = Field(default_factory=list)
    include_minority_interest: bool = Field(default=True)


class IntercompanyTransaction(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    id: Optional[str] = None
    from_entity_id: str
    to_entity_id: str
    transaction_type: str = Field(..., description="E.g. sale, loan, dividend, management_fee, royalty")
    amount: float = Field(..., gt=0)
    currency: str = Field(default="NGN", max_length=3)
    exchange_rate: float = Field(default=1.0, gt=0)
    transaction_date: datetime
    description: str = Field(..., min_length=1, max_length=500)
    elimination_account_code: Optional[str] = Field(default=None, pattern=r"^\d{4,5}$")

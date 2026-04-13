# Author: Quadri Atharu
"""Journal entry Pydantic v2 models."""

from __future__ import annotations

from datetime import datetime
from enum import Enum
from typing import Optional

from pydantic import BaseModel, Field, ConfigDict, model_validator


class JournalStatus(str, Enum):
    DRAFT = "DRAFT"
    SUBMITTED = "SUBMITTED"
    APPROVED = "APPROVED"
    POSTED = "POSTED"
    REVERSED = "REVERSED"
    VOID = "VOID"


class JournalLineBase(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    account_code: str = Field(..., pattern=r"^\d{4,5}$")
    description: str = Field(..., min_length=1, max_length=500)
    debit: float = Field(default=0.0, ge=0)
    credit: float = Field(default=0.0, ge=0)
    reference: Optional[str] = Field(default=None, max_length=100)
    cost_center: Optional[str] = Field(default=None, max_length=100)
    tax_code: Optional[str] = Field(default=None, max_length=20)

    @model_validator(mode="after")
    def debit_or_credit(self) -> "JournalLineBase":
        if self.debit > 0 and self.credit > 0:
            raise ValueError("A journal line cannot have both debit and credit amounts")
        if self.debit == 0 and self.credit == 0:
            raise ValueError("A journal line must have either a debit or credit amount")
        return self


class JournalLineCreate(JournalLineBase):
    pass


class JournalEntryBase(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    entry_date: datetime
    description: str = Field(..., min_length=1, max_length=500)
    reference: Optional[str] = Field(default=None, max_length=100)
    source_type: Optional[str] = Field(default=None, max_length=50, description="E.g. invoice, bill, adjustment")
    source_id: Optional[str] = Field(default=None, description="ID of the source document")
    is_adjusting: bool = Field(default=False, description="Whether this is an adjusting entry")
    is_reversing: bool = Field(default=False, description="Whether this reverses a prior entry")
    reversing_entry_id: Optional[str] = Field(default=None)


class JournalEntryCreate(JournalEntryBase):
    company_id: str
    lines: list[JournalLineCreate] = Field(..., min_length=2)

    @model_validator(mode="after")
    def entry_must_balance(self) -> "JournalEntryCreate":
        total_debit = sum(line.debit for line in self.lines)
        total_credit = sum(line.credit for line in self.lines)
        if abs(total_debit - total_credit) > 0.01:
            raise ValueError(
                f"Journal entry does not balance: debits={total_debit}, credits={total_credit}"
            )
        return self


class JournalEntryResponse(JournalEntryBase):
    model_config = ConfigDict(from_attributes=True)

    id: str
    company_id: str
    entry_number: str
    status: JournalStatus = JournalStatus.DRAFT
    total_debit: float = 0.0
    total_credit: float = 0.0
    lines: list[JournalLineCreate] = []
    created_by: Optional[str] = None
    approved_by: Optional[str] = None
    created_at: datetime
    updated_at: datetime

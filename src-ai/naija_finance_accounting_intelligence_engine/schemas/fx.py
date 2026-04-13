# Author: Quadri Atharu
"""Foreign exchange and multi-currency Pydantic v2 models."""

from __future__ import annotations

from datetime import datetime
from typing import List, Optional

from pydantic import BaseModel, Field, ConfigDict


class ExchangeRate(BaseModel):
    base_currency: str = Field(default="NGN", max_length=3)
    quote_currency: str = Field(default="USD", max_length=3)
    rate: float = Field(..., gt=0)
    source: str = Field(default="CBN", max_length=50, description="Rate source: CBN, parallel, custom")
    effective_date: datetime
    created_at: datetime = Field(default_factory=datetime.now)


class FxTransaction(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    company_id: str
    transaction_date: datetime
    base_currency: str = Field(default="NGN", max_length=3)
    foreign_currency: str = Field(..., max_length=3)
    foreign_amount: float = Field(..., gt=0)
    exchange_rate: float = Field(..., gt=0)
    base_amount: float = Field(default=0.0, description="Computed as foreign_amount * exchange_rate")
    transaction_type: str = Field(default="spot", pattern=r"^(spot|forward|hedge)$")
    reference: Optional[str] = None
    description: Optional[str] = None

    def compute_base_amount(self) -> float:
        self.base_amount = round(self.foreign_amount * self.exchange_rate, 2)
        return self.base_amount


class FxGainLossEntry(BaseModel):
    id: Optional[str] = None
    company_id: str
    account_code: str = Field(..., pattern=r"^\d{4,5}$")
    transaction_date: datetime
    original_rate: float
    closing_rate: float
    original_base_amount: float
    revalued_base_amount: float
    fx_gain_loss: float = Field(default=0.0, description="Positive = gain, negative = loss")
    gain_loss_type: str = Field(default="unrealized", pattern=r"^(realized|unrealized)$")
    currency: str = Field(default="USD", max_length=3)
    reference: Optional[str] = None


class MultiCurrencyRequest(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    company_id: str
    reporting_currency: str = Field(default="NGN", max_length=3)
    period_end: datetime
    transactions: List[FxTransaction] = Field(default_factory=list)
    closing_rates: List[ExchangeRate] = Field(default_factory=list)
    compute_unrealized: bool = Field(default=True, description="Compute unrealized FX gains/losses at period end")

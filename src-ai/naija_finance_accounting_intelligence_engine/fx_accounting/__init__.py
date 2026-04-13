"""Foreign exchange accounting module for multi-currency operations.

Author: Quadri Atharu

Provides multi-currency transaction processing, financial statement
translation, FX exposure computation, exchange rate tracking, and
FX gain/loss recognition per IAS 21 (The Effects of Changes in
Foreign Exchange Rates) with Nigeria-specific NGN rate support.
"""

from .multi_currency import (
    process_fx_transaction,
    translate_financial_statements,
    compute_fx_exposure,
)
from .exchange_rates import (
    get_cbn_rate,
    get_parallel_market_rate,
    get_average_rate,
    get_closing_rate,
)
from .fx_gain_loss import (
    compute_realized_fx_gain_loss,
    compute_unrealized_fx_gain_loss,
    recognize_fx_adjustment,
)

__all__ = [
    "process_fx_transaction",
    "translate_financial_statements",
    "compute_fx_exposure",
    "get_cbn_rate",
    "get_parallel_market_rate",
    "get_average_rate",
    "get_closing_rate",
    "compute_realized_fx_gain_loss",
    "compute_unrealized_fx_gain_loss",
    "recognize_fx_adjustment",
]

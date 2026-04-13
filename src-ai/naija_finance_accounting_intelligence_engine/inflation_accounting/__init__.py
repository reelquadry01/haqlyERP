"""Inflation accounting module per IAS 29.

Author: Quadri Atharu

Provides hyperinflation detection, financial statement restatement,
historical cost adjustment, and monetary gain/loss computation
for economies experiencing high inflation, with Nigeria-specific
threshold checks and CPI-based adjustment factors.
"""

from .inflation_adjusted import (
    is_hyperinflationary,
    adjust_financial_statements,
    restate_historical_cost,
    compute_monetary_gain_loss,
)
from .real_nominal import (
    compute_real_value,
    generate_real_vs_nominal_report,
    deflate_series,
)

__all__ = [
    "is_hyperinflationary",
    "adjust_financial_statements",
    "restate_historical_cost",
    "compute_monetary_gain_loss",
    "compute_real_value",
    "generate_real_vs_nominal_report",
    "deflate_series",
]

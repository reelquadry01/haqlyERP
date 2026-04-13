"""Real vs nominal financial comparison and series deflation.

Author: Quadri Atharu

Converts nominal financial amounts to real values by stripping out
inflation effects, generates comparative real vs nominal reports,
and deflates time series data to a common base period for meaningful
cross-period analysis in the Nigerian economic context.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class RealValue:
    nominal_amount: Decimal
    inflation_rate: Decimal
    real_amount: Decimal
    purchasing_power_loss: Decimal


@dataclass
class RealVsNominalReport:
    line_items: dict[str, dict[str, Decimal]] = field(default_factory=dict)
    total_nominal: Decimal = Decimal("0")
    total_real: Decimal = Decimal("0")
    total_purchasing_power_loss: Decimal = Decimal("0")
    inflation_rate: Decimal = Decimal("0")


@dataclass
class DeflatedSeries:
    base_period: str
    nominal_values: dict[str, Decimal]
    real_values: dict[str, Decimal]
    deflators: dict[str, Decimal]


def compute_real_value(
    nominal_amount: Decimal,
    inflation_rate: Decimal,
) -> RealValue:
    """Convert a nominal amount to its real value by removing the
    effect of inflation.

    Real Value = Nominal Amount / (1 + inflation_rate)

    This adjusts for the loss of purchasing power so that financial
    amounts are comparable across periods with different price levels.

    Args:
        nominal_amount: The nominal (unadjusted) amount.
        inflation_rate: The inflation rate as a percentage (e.g. 28.9 for 28.9%).

    Returns:
        RealValue with the inflation-adjusted amount and purchasing power loss.
    """
    nominal_amount = _d(nominal_amount)
    inflation_rate = _d(inflation_rate)
    rate_decimal = inflation_rate / Decimal("100")
    deflator = Decimal("1") + rate_decimal

    if deflator == Decimal("0"):
        real_amount = nominal_amount
    else:
        real_amount = (nominal_amount / deflator).quantize(TWO_PLACES)

    purchasing_power_loss = nominal_amount - real_amount

    return RealValue(
        nominal_amount=nominal_amount,
        inflation_rate=inflation_rate,
        real_amount=real_amount,
        purchasing_power_loss=purchasing_power_loss,
    )


def generate_real_vs_nominal_report(
    nominal_statements: dict[str, Any],
    inflation_data: dict[str, Decimal],
) -> RealVsNominalReport:
    """Generate a comparative real vs nominal financial report.

    Takes nominal financial statements and inflation data for each
    line item's relevant period, and computes the real value for
    every item, producing a side-by-side comparison.

    Args:
        nominal_statements: Dict with section names ('revenue', 'expenses',
                           etc.) mapping to dicts of line item name -> amount.
        inflation_data: Dict mapping period or line item key to inflation
                       rate (percentage). Falls back to 'default' key.

    Returns:
        RealVsNominalReport with nominal, real, and loss for each item.
    """
    default_inflation = _d(inflation_data.get("default", Decimal("0")))
    line_items: dict[str, dict[str, Decimal]] = {}
    total_nominal = Decimal("0")
    total_real = Decimal("0")
    total_loss = Decimal("0")

    for section, items in nominal_statements.items():
        if not isinstance(items, dict):
            continue
        for item_name, amount in items.items():
            nominal = _d(amount)
            rate = _d(inflation_data.get(item_name, inflation_data.get(section, default_inflation)))
            rv = compute_real_value(nominal, rate)

            line_items[item_name] = {
                "nominal": nominal,
                "real": rv.real_amount,
                "purchasing_power_loss": rv.purchasing_power_loss,
                "inflation_rate": rate,
            }

            total_nominal += nominal
            total_real += rv.real_amount
            total_loss += rv.purchasing_power_loss

    return RealVsNominalReport(
        line_items=line_items,
        total_nominal=total_nominal,
        total_real=total_real,
        total_purchasing_power_loss=total_loss,
        inflation_rate=default_inflation,
    )


def deflate_series(
    nominal_series: dict[str, Decimal],
    base_period: str,
) -> DeflatedSeries:
    """Deflate a nominal time series to real values relative to a base period.

    Each nominal value is divided by the cumulative inflation factor
    from the base period, producing a series in constant purchasing
    power terms.

    Args:
        nominal_series: Dict mapping period labels (e.g. '2024-Q1')
                       to nominal amounts.
        base_period: The base period label. Must be a key in nominal_series.

    Returns:
        DeflatedSeries with nominal, real, and deflator for each period.

    Raises:
        ValueError: If base_period is not in nominal_series.
    """
    if base_period not in nominal_series:
        raise ValueError(f"Base period '{base_period}' not found in nominal series")

    base_value = _d(nominal_series[base_period])
    real_values: dict[str, Decimal] = {}
    deflators: dict[str, Decimal] = {}

    for period, nominal in nominal_series.items():
        nominal = _d(nominal)
        if base_value == Decimal("0"):
            deflator = Decimal("1")
        else:
            deflator = nominal / base_value
        real = base_value if period == base_period else (base_value * (Decimal("1") / deflator)).quantize(TWO_PLACES) if deflator != Decimal("0") else base_value

        deflators[period] = deflator.quantize(TWO_PLACES)
        real_values[period] = real

    return DeflatedSeries(
        base_period=base_period,
        nominal_values={k: _d(v) for k, v in nominal_series.items()},
        real_values=real_values,
        deflators=deflators,
    )

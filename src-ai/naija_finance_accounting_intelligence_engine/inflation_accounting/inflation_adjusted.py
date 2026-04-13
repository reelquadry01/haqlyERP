"""IAS 29 hyperinflation-adjusted financial statement restatement.

Author: Quadri Atharu

Implements IAS 29 requirements for financial reporting in hyperinflationary
economies. Detects hyperinflation conditions, restates financial statements
using a general price index, adjusts historical costs, and computes
monetary gain/loss arising from holding net monetary positions during
inflationary periods.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


HYPERINFLATION_CUMULATIVE_THRESHOLD = Decimal("100")
HYPERINFLATION_ANNUAL_THRESHOLD = Decimal("26")


@dataclass
class HyperinflationCheck:
    is_hyperinflationary: bool
    cumulative_3yr_inflation: Decimal
    annual_inflation: Decimal
    indicators_met: list[str]
    indicators_not_met: list[str]


@dataclass
class AdjustedStatement:
    original: dict[str, Decimal]
    adjusted: dict[str, Decimal]
    adjustment_factor: Decimal
    adjustments_detail: dict[str, Decimal] = field(default_factory=dict)


@dataclass
class RestatedAmount:
    original_amount: Decimal
    restated_amount: Decimal
    inflation_factor: Decimal
    difference: Decimal


@dataclass
class MonetaryGainLoss:
    net_monetary_position: Decimal
    inflation_rate: Decimal
    monetary_gain: Decimal
    monetary_loss: Decimal
    net_gain_loss: Decimal


def is_hyperinflationary(economy_data: dict[str, Any]) -> HyperinflationCheck:
    """Determine if an economy is hyperinflationary per IAS 29.

    IAS 29 Appendix A lists indicators of hyperinflation:
    1. Cumulative inflation over 3 years approaching or exceeding 100%
    2. General population prefers to keep wealth in non-monetary assets
    3. Prices are quoted in a stable foreign currency
    4. Credit sales priced to compensate for expected purchasing power loss
    5. Interest rates, wages, and prices indexed to a price index
    6. Cumulative inflation rate over 3 years > 100%

    For Nigeria, uses NBS CPI data to check thresholds.

    Args:
        economy_data: Dict with 'cumulative_3yr_inflation', 'annual_inflation',
                     'population_prefers_foreign_currency',
                     'prices_quoted_in_foreign_currency',
                     'credit_sales_compensate_inflation',
                     'wages_indexed', 'interest_rates_indexed'.

    Returns:
        HyperinflationCheck with boolean result and indicator details.
    """
    cumulative_3yr = _d(economy_data.get("cumulative_3yr_inflation", 0))
    annual = _d(economy_data.get("annual_inflation", 0))

    indicators_met: list[str] = []
    indicators_not_met: list[str] = []

    if cumulative_3yr >= HYPERINFLATION_CUMULATIVE_THRESHOLD:
        indicators_met.append("Cumulative 3-year inflation >= 100%")
    else:
        indicators_not_met.append(f"Cumulative 3-year inflation ({cumulative_3yr}%) < 100%")

    if annual >= HYPERINFLATION_ANNUAL_THRESHOLD:
        indicators_met.append("Annual inflation >= 26% (doubling in < 3 years)")
    else:
        indicators_not_met.append(f"Annual inflation ({annual}%) < 26%")

    bool_indicators = [
        ("population_prefers_foreign_currency", "Population prefers foreign currency"),
        ("prices_quoted_in_foreign_currency", "Prices quoted in foreign currency"),
        ("credit_sales_compensate_inflation", "Credit sales compensate for inflation"),
        ("wages_indexed", "Wages indexed to price index"),
        ("interest_rates_indexed", "Interest rates indexed to price index"),
    ]

    for key, description in bool_indicators:
        if economy_data.get(key, False):
            indicators_met.append(description)
        else:
            indicators_not_met.append(description)

    is_hyper = len(indicators_met) >= 3 or cumulative_3yr >= HYPERINFLATION_CUMULATIVE_THRESHOLD

    return HyperinflationCheck(
        is_hyperinflationary=is_hyper,
        cumulative_3yr_inflation=cumulative_3yr,
        annual_inflation=annual,
        indicators_met=indicators_met,
        indicators_not_met=indicators_not_met,
    )


def adjust_financial_statements(
    statements: dict[str, Any],
    inflation_index: dict[str, Decimal],
) -> AdjustedStatement:
    """Adjust financial statements for hyperinflation per IAS 29.

    Non-monetary items are restated using the conversion factor derived
    from the general price index. Monetary items are NOT restated but
    their purchasing power loss/gain is recognized separately.

    Balance sheet: non-monetary assets restated; monetary items at nominal.
    Income statement: all items restated from transaction date to
    reporting date using the price index.

    Args:
        statements: Dict with 'balance_sheet' and 'income_statement',
                   each containing line items as dicts of name -> amount.
                   Items should have 'type' field: 'monetary' or 'non_monetary'.
        inflation_index: Dict mapping date/period to index value, with
                        'base_period' and 'current_period' keys.

    Returns:
        AdjustedStatement with original, adjusted, and adjustment details.
    """
    base_index = _d(inflation_index.get("base_period", Decimal("100")))
    current_index = _d(inflation_index.get("current_period", Decimal("100")))

    if base_index == Decimal("0"):
        adjustment_factor = Decimal("1")
    else:
        adjustment_factor = current_index / base_index

    original: dict[str, Decimal] = {}
    adjusted: dict[str, Decimal] = {}
    adjustments_detail: dict[str, Decimal] = {}

    for section in ["balance_sheet", "income_statement"]:
        section_data = statements.get(section, {})
        for key, value in section_data.items():
            if isinstance(value, dict):
                item_type = value.get("type", "non_monetary")
                amount = _d(value.get("amount", 0))
            else:
                item_type = "non_monetary"
                amount = _d(value)

            original[key] = amount

            if item_type == "monetary":
                adjusted[key] = amount
                adjustments_detail[key] = Decimal("0")
            else:
                restated = (amount * adjustment_factor).quantize(TWO_PLACES)
                adjusted[key] = restated
                adjustments_detail[key] = restated - amount

    return AdjustedStatement(
        original=original,
        adjusted=adjusted,
        adjustment_factor=adjustment_factor,
        adjustments_detail=adjustments_detail,
    )


def restate_historical_cost(
    asset: dict[str, Any],
    inflation_factor: Decimal,
) -> RestatedAmount:
    """Restate a historical cost amount using the inflation factor.

    Per IAS 29, non-monetary assets carried at historical cost are
    restated by applying the ratio of the general price index at
    the reporting date to the index at the acquisition date.

    Args:
        asset: Dict with 'historical_cost', 'acquisition_date',
              'description', 'carrying_amount'.
        inflation_factor: The conversion factor (current index / base index).

    Returns:
        RestatedAmount with original, restated, and difference.
    """
    historical_cost = _d(asset.get("historical_cost", 0))
    inflation_factor = _d(inflation_factor)

    restated = (historical_cost * inflation_factor).quantize(TWO_PLACES)
    difference = restated - historical_cost

    return RestatedAmount(
        original_amount=historical_cost,
        restated_amount=restated,
        inflation_factor=inflation_factor,
        difference=difference,
    )


def compute_monetary_gain_loss(
    monetary_items: list[dict[str, Any]],
    inflation_rate: Decimal,
) -> MonetaryGainLoss:
    """Compute the monetary gain or loss from holding net monetary
    positions during an inflationary period.

    Per IAS 29, a gain on net monetary liabilities or a loss on net
    monetary assets arises during hyperinflation. The gain/loss is
    recognized in profit or loss.

    Args:
        monetary_items: List of dicts with 'account', 'amount',
                       'type' (monetary_asset/monetary_liability).
        inflation_rate: The inflation rate for the period (as percentage).

    Returns:
        MonetaryGainLoss with net position, gain, loss, and net result.
    """
    inflation_rate = _d(inflation_rate)
    rate_decimal = inflation_rate / Decimal("100")

    total_monetary_assets = Decimal("0")
    total_monetary_liabilities = Decimal("0")

    for item in monetary_items:
        amount = _d(item.get("amount", 0))
        item_type = item.get("type", "monetary_asset")

        if item_type == "monetary_asset":
            total_monetary_assets += amount
        else:
            total_monetary_liabilities += amount

    net_monetary_position = total_monetary_assets - total_monetary_liabilities

    loss_on_assets = (total_monetary_assets * rate_decimal).quantize(TWO_PLACES)
    gain_on_liabilities = (total_monetary_liabilities * rate_decimal).quantize(TWO_PLACES)
    net_gain_loss = gain_on_liabilities - loss_on_assets

    return MonetaryGainLoss(
        net_monetary_position=net_monetary_position,
        inflation_rate=inflation_rate,
        monetary_gain=gain_on_liabilities,
        monetary_loss=loss_on_assets,
        net_gain_loss=net_gain_loss,
    )

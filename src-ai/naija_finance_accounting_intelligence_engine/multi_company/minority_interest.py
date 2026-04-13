"""Non-controlling interest (minority interest) computation.

Author: Quadri Atharu

Computes NCI shares of subsidiary net income and equity for
consolidated financial statement presentation per IFRS 10
and IAS 1 requirements. Supports both partial and full
goodwill methods for NCI measurement.
"""

from __future__ import annotations

from dataclasses import dataclass
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class NCIResult:
    nci_amount: Decimal
    subsidiary_net_income: Decimal
    minority_percentage: Decimal
    method: str = "partial_goodwill"


@dataclass
class NCIBalanceSheet:
    nci_equity: Decimal
    subsidiary_equity: Decimal
    minority_percentage: Decimal
    breakdown: dict[str, Decimal]


@dataclass
class IncomeAllocation:
    parent_share: Decimal
    nci_share: Decimal
    total_income: Decimal
    parent_pct: Decimal
    nci_pct: Decimal


def compute_minority_interest(
    subsidiary_net_income: Decimal,
    minority_percentage: Decimal,
) -> NCIResult:
    """Compute the non-controlling interest share of subsidiary net income.

    Under IFRS 10, NCI is allocated its proportionate share of the
    subsidiary's profit or loss and other comprehensive income.

    Args:
        subsidiary_net_income: The subsidiary's total net income for the period.
        minority_percentage: The NCI ownership percentage (0-100).

    Returns:
        NCIResult with computed NCI amount and supporting details.
    """
    subsidiary_net_income = _d(subsidiary_net_income)
    minority_percentage = _d(minority_percentage)
    nci_amount = (subsidiary_net_income * minority_percentage / Decimal("100")).quantize(TWO_PLACES)

    return NCIResult(
        nci_amount=nci_amount,
        subsidiary_net_income=subsidiary_net_income,
        minority_percentage=minority_percentage,
    )


def compute_nci_on_balance_sheet(
    subsidiary_equity: dict[str, Decimal],
    minority_percentage: Decimal,
) -> NCIBalanceSheet:
    """Compute the non-controlling interest equity on the consolidated
    balance sheet.

    NCI is presented within equity but separately from the parent's
    equity per IAS 1. The NCI share covers all equity components:
    share capital, share premium, retained earnings, and other reserves.

    Args:
        subsidiary_equity: Dict of equity component names to amounts
                          (e.g. {'share_capital': 500000, 'retained_earnings': 200000}).
        minority_percentage: The NCI ownership percentage (0-100).

    Returns:
        NCIBalanceSheet with total NCI equity and per-component breakdown.
    """
    minority_percentage = _d(minority_percentage)
    total_equity = Decimal("0")
    breakdown: dict[str, Decimal] = {}

    for component, amount in subsidiary_equity.items():
        component_amount = _d(amount)
        total_equity += component_amount
        nci_component = (component_amount * minority_percentage / Decimal("100")).quantize(TWO_PLACES)
        breakdown[component] = nci_component

    nci_equity = (total_equity * minority_percentage / Decimal("100")).quantize(TWO_PLACES)

    return NCIBalanceSheet(
        nci_equity=nci_equity,
        subsidiary_equity=total_equity,
        minority_percentage=minority_percentage,
        breakdown=breakdown,
    )


def allocate_income(
    subsidiary_net_income: Decimal,
    parent_pct: Decimal,
    minority_pct: Decimal,
) -> IncomeAllocation:
    """Allocate subsidiary net income between parent equity holders and
    non-controlling interests.

    The allocation must always sum to 100% of subsidiary net income.
    Used for the consolidated income statement attribution of profit.

    Args:
        subsidiary_net_income: Subsidiary's net income for the period.
        parent_pct: Parent ownership percentage (0-100).
        minority_pct: NCI ownership percentage (0-100), should equal 100 - parent_pct.

    Returns:
        IncomeAllocation with parent and NCI shares of income.

    Raises:
        ValueError: If parent_pct + minority_pct != 100.
    """
    parent_pct = _d(parent_pct)
    minority_pct = _d(minority_pct)
    subsidiary_net_income = _d(subsidiary_net_income)

    if parent_pct + minority_pct != Decimal("100"):
        raise ValueError(
            f"parent_pct ({parent_pct}) + minority_pct ({minority_pct}) must equal 100"
        )

    parent_share = (subsidiary_net_income * parent_pct / Decimal("100")).quantize(TWO_PLACES)
    nci_share = (subsidiary_net_income * minority_pct / Decimal("100")).quantize(TWO_PLACES)

    return IncomeAllocation(
        parent_share=parent_share,
        nci_share=nci_share,
        total_income=subsidiary_net_income,
        parent_pct=parent_pct,
        nci_pct=minority_pct,
    )

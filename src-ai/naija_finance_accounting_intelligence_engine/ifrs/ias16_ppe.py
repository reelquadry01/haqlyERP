"""IAS 16 Property, Plant and Equipment.

Author: Quadri Atharu

Implements IAS 16 expenditure classification, depreciation computation,
revaluation surplus/deficit, and impairment testing for property,
plant, and equipment.
"""

from __future__ import annotations

from dataclasses import dataclass
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class ExpenditureClassification:
    amount: Decimal
    classification: str
    rationale: str


@dataclass
class DepreciationResult:
    asset_name: str
    method: str
    depreciable_amount: Decimal
    annual_depreciation: Decimal
    useful_life: int
    residual_value: Decimal


@dataclass
class RevaluationResult:
    asset_name: str
    carrying_amount_before: Decimal
    fair_value: Decimal
    revaluation_surplus: Decimal
    revaluation_deficit: Decimal
    adjustment_to_income: Decimal
    adjustment_to_oci: Decimal


@dataclass
class ImpairmentPPEResult:
    asset_name: str
    carrying_amount: Decimal
    recoverable_amount: Decimal
    impairment_loss: Decimal


def classify_expenditure(
    amount: Decimal,
    expenditure_type: str,
) -> ExpenditureClassification:
    """Classify expenditure as capital or revenue per IAS 16.

    Capital expenditure: enhances future economic benefits beyond
    original standard of performance (additions, improvements,
    major replacements that extend useful life).
    Revenue expenditure: maintains original standard of performance
    (repairs, maintenance, minor replacements).

    Args:
        amount: The expenditure amount.
        expenditure_type: Type description (e.g. 'new_asset', 'extension',
                         'major_repair_extend_life', 'routine_maintenance',
                         'replacement_improve', 'replacement_same',
                         'addition', 'repair').

    Returns:
        ExpenditureClassification with capital/revenue classification.
    """
    amount = _d(amount)
    expenditure_type = expenditure_type.lower()

    capital_types = {
        "new_asset", "extension", "addition", "major_repair_extend_life",
        "replacement_improve", "improvement", "betterment",
        "major_overhaul_extend_life", "upgrade",
    }

    revenue_types = {
        "routine_maintenance", "repair", "replacement_same",
        "minor_repair", "servicing", "cleaning",
    }

    if expenditure_type in capital_types:
        return ExpenditureClassification(
            amount=amount,
            classification="Capital",
            rationale=f"Expenditure enhances future economic benefits beyond original standard ({expenditure_type})",
        )
    elif expenditure_type in revenue_types:
        return ExpenditureClassification(
            amount=amount,
            classification="Revenue",
            rationale=f"Expenditure maintains original standard of performance ({expenditure_type})",
        )
    else:
        return ExpenditureClassification(
            amount=amount,
            classification="Revenue",
            rationale=f"Default classification as revenue expenditure for '{expenditure_type}' - verify if future benefits are enhanced",
        )


def compute_depreciation(
    asset: dict[str, Any],
    method: str,
    life: int,
    residual: Decimal,
) -> DepreciationResult:
    """Compute depreciation for a PPE asset.

    Supports: Straight-line, Declining balance, Units of production.

    Args:
        asset: Dict with 'name', 'cost', 'accumulated_depreciation',
              'total_estimated_units', 'units_this_period'.
        method: Depreciation method ('straight_line', 'declining_balance',
               'units_of_production').
        life: Useful life in years.
        residual: Estimated residual value.

    Returns:
        DepreciationResult with annual depreciation amount.
    """
    cost = _d(asset.get("cost", 0))
    residual = _d(residual)
    life = max(life, 1)
    depreciable = cost - residual

    method = method.lower()

    if method == "straight_line":
        annual = (depreciable / Decimal(str(life))).quantize(TWO_PLACES)
    elif method == "declining_balance":
        rate = Decimal("2") / Decimal(str(life))
        book_value = cost - _d(asset.get("accumulated_depreciation", 0))
        annual = (book_value * rate).quantize(TWO_PLACES)
        if book_value - annual < residual:
            annual = (book_value - residual).quantize(TWO_PLACES)
    elif method == "units_of_production":
        total_units = _d(asset.get("total_estimated_units", 1))
        units_this = _d(asset.get("units_this_period", 0))
        rate = depreciable / total_units if total_units > 0 else Decimal("0")
        annual = (units_this * rate).quantize(TWO_PLACES)
    else:
        annual = (depreciable / Decimal(str(life))).quantize(TWO_PLACES)

    return DepreciationResult(
        asset_name=asset.get("name", ""),
        method=method,
        depreciable_amount=depreciable,
        annual_depreciation=annual,
        useful_life=life,
        residual_value=residual,
    )


def compute_revaluation(
    asset: dict[str, Any],
    fair_value: Decimal,
    accumulated_dep: Decimal,
) -> RevaluationResult:
    """Compute revaluation surplus or deficit per IAS 16.

    Revaluation surplus (gain): credited to OCI (Revaluation Surplus)
    unless reversing a previous deficit charged to profit.
    Revaluation deficit (loss): charged to profit unless reversing
    a previous surplus credited to OCI.

    Args:
        asset: Dict with 'name', 'cost'.
        fair_value: The new fair value of the asset.
        accumulated_dep: Accumulated depreciation at revaluation date.

    Returns:
        RevaluationResult with surplus/deficit and P&L/OCI allocation.
    """
    cost = _d(asset.get("cost", 0))
    accumulated_dep = _d(accumulated_dep)
    fair_value = _d(fair_value)

    carrying_before = cost - accumulated_dep
    difference = fair_value - carrying_before

    if difference >= Decimal("0"):
        surplus = difference
        deficit = Decimal("0")
        adjustment_oci = surplus
        adjustment_income = Decimal("0")
    else:
        surplus = Decimal("0")
        deficit = abs(difference)
        adjustment_oci = Decimal("0")
        adjustment_income = deficit

    return RevaluationResult(
        asset_name=asset.get("name", ""),
        carrying_amount_before=carrying_before,
        fair_value=fair_value,
        revaluation_surplus=surplus,
        revaluation_deficit=deficit,
        adjustment_to_income=adjustment_income,
        adjustment_to_oci=adjustment_oci,
    )


def compute_impairment_ppe(
    asset: dict[str, Any],
    recoverable_amount: Decimal,
) -> ImpairmentPPEResult:
    """Compute impairment loss for PPE per IAS 36.

    An impairment loss occurs when the carrying amount exceeds the
    recoverable amount (higher of fair value less costs of disposal
    and value in use).

    Args:
        asset: Dict with 'name', 'cost', 'accumulated_depreciation'.
        recoverable_amount: The recoverable amount per IAS 36.

    Returns:
        ImpairmentPPEResult with impairment loss if applicable.
    """
    cost = _d(asset.get("cost", 0))
    acc_dep = _d(asset.get("accumulated_depreciation", 0))
    carrying = cost - acc_dep
    recoverable = _d(recoverable_amount)

    impairment = max(Decimal("0"), carrying - recoverable)

    return ImpairmentPPEResult(
        asset_name=asset.get("name", ""),
        carrying_amount=carrying,
        recoverable_amount=recoverable,
        impairment_loss=impairment,
    )

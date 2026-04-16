"""IAS 12 Income Taxes - deferred tax computation.

Author: Quadri Atharu

Implements IAS 12 deferred tax liability and asset computation,
current tax recognition, total tax expense calculation, and
DTA recognition criteria based on probable future taxable profit.

Note: Nigerian statutory CIT rate is 25% for large companies per
Tax Reform Acts 2025 (effective 2026). Use 15% for medium companies
(turnover NGN 50M-250M) and 0% for small companies (<= NGN 50M).
"""

from __future__ import annotations

from dataclasses import dataclass
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class DeferredTaxResult:
    temporary_differences: Decimal
    tax_rate: Decimal
    deferred_tax_amount: Decimal


@dataclass
class CurrentTaxResult:
    taxable_income: Decimal
    tax_rate: Decimal
    current_tax: Decimal


@dataclass
class TaxExpenseResult:
    current_tax: Decimal
    deferred_tax_movement: Decimal
    total_tax_expense: Decimal


@dataclass
class RecognitionValidation:
    dta_amount: Decimal
    probability_of_profit: bool
    can_recognize: bool
    rationale: str


def compute_deferred_tax_liability(
    taxable_temporary_differences: Decimal,
    rate: Decimal,
) -> DeferredTaxResult:
    """Compute deferred tax liability from taxable temporary differences.

    Taxable temporary differences result in deferred tax liabilities -
    amounts of tax payable in future periods when the carrying amount
    of assets/liabilities is recovered/settled.

    Args:
        taxable_temporary_differences: Total taxable temporary differences.
        rate: Applicable tax rate as percentage.

    Returns:
        DeferredTaxResult with DTL amount.
    """
    differences = _d(taxable_temporary_differences)
    rate = _d(rate)
    dtl = (differences * rate / Decimal("100")).quantize(TWO_PLACES)

    return DeferredTaxResult(
        temporary_differences=differences,
        tax_rate=rate,
        deferred_tax_amount=dtl,
    )


def compute_deferred_tax_asset(
    deductible_temporary_differences: Decimal,
    rate: Decimal,
) -> DeferredTaxResult:
    """Compute deferred tax asset from deductible temporary differences.

    Deductible temporary differences result in deferred tax assets -
    amounts of tax recoverable in future periods when the carrying
    amount of assets/liabilities is recovered/settled.

    Args:
        deductible_temporary_differences: Total deductible temporary differences.
        rate: Applicable tax rate as percentage.

    Returns:
        DeferredTaxResult with DTA amount.
    """
    differences = _d(deductible_temporary_differences)
    rate = _d(rate)
    dta = (differences * rate / Decimal("100")).quantize(TWO_PLACES)

    return DeferredTaxResult(
        temporary_differences=differences,
        tax_rate=rate,
        deferred_tax_amount=dta,
    )


def recognize_current_tax(
    taxable_income: Decimal,
    rate: Decimal,
) -> CurrentTaxResult:
    """Recognize current tax expense based on taxable income.

    Args:
        taxable_income: The taxable income for the period.
        rate: Applicable tax rate as percentage.

    Returns:
        CurrentTaxResult with current tax amount.
    """
    income = _d(taxable_income)
    rate = _d(rate)
    tax = (income * rate / Decimal("100")).quantize(TWO_PLACES)

    return CurrentTaxResult(
        taxable_income=income,
        tax_rate=rate,
        current_tax=tax,
    )


def compute_tax_expense(
    current_tax: Decimal,
    deferred_tax_movement: Decimal,
) -> TaxExpenseResult:
    """Compute total tax expense per IAS 12.

    Tax expense = Current tax + Deferred tax movement
    (Deferred tax movement = DTL increase - DTA increase, or net change)

    Args:
        current_tax: Current tax for the period.
        deferred_tax_movement: Net deferred tax movement (positive = expense,
                              negative = benefit).

    Returns:
        TaxExpenseResult with total tax expense.
    """
    current = _d(current_tax)
    deferred = _d(deferred_tax_movement)
    total = (current + deferred).quantize(TWO_PLACES)

    return TaxExpenseResult(
        current_tax=current,
        deferred_tax_movement=deferred,
        total_tax_expense=total,
    )


def validate_recognition(
    dta: Decimal,
    probability_of_profit: bool,
) -> RecognitionValidation:
    """Validate whether a deferred tax asset can be recognized.

    Per IAS 12.24, a DTA shall be recognized only to the extent
    that it is probable that taxable profit will be available
    against which the deductible temporary difference can be utilized.

    Args:
        dta: The computed DTA amount.
        probability_of_profit: Whether future taxable profit is probable.

    Returns:
        RecognitionValidation with recognition decision and rationale.
    """
    dta = _d(dta)

    if dta <= Decimal("0"):
        return RecognitionValidation(
            dta_amount=dta,
            probability_of_profit=probability_of_profit,
            can_recognize=True,
            rationale="No DTA to recognize (amount is zero or negative)",
        )

    if probability_of_profit:
        return RecognitionValidation(
            dta_amount=dta,
            probability_of_profit=True,
            can_recognize=True,
            rationale="DTA recognized - probable future taxable profit exists per IAS 12.24",
        )

    return RecognitionValidation(
        dta_amount=dta,
        probability_of_profit=False,
        can_recognize=False,
        rationale="DTA not recognized - future taxable profit not probable per IAS 12.24. Disclose in notes per IAS 12.88",
    )

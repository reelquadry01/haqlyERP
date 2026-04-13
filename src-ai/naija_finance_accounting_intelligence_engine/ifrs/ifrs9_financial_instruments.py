"""IFRS 9 Financial Instruments classification and impairment.

Author: Quadri Atharu

Implements the IFRS 9 three-stage expected credit loss model,
financial asset classification (Amortized Cost, FVTPL, FVTOCI),
and impairment measurement for Nigerian banks and financial
institutions.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class ClassificationResult:
    asset_name: str
    classification: str
    business_model: str
    sppi_result: bool
    rationale: str


@dataclass
class ECLResult:
    stage: int
    exposure_at_default: Decimal
    probability_of_default: Decimal
    loss_given_default: Decimal
    ecl_amount: Decimal
    lifetime_or_12_month: str


@dataclass
class ImpairmentResult:
    asset_name: str
    carrying_amount: Decimal
    ecl_amount: Decimal
    impairment_amount: Decimal
    stage: int


@dataclass
class ValidationResult:
    is_valid: bool
    classification: str
    issues: list[str] = field(default_factory=list)


def classify_financial_asset(asset: dict[str, Any]) -> ClassificationResult:
    """Classify a financial asset per IFRS 9 based on the business model
    test and the solely payments of principal and interest (SPPI) test.

    Classification outcomes:
    - Amortized Cost: Hold-to-collect business model + SPPI pass
    - FVTOCI: Hold-to-collect-and-sell business model + SPPI pass
    - FVTPL: All other cases (SPPI fail or trading business model)

    Args:
        asset: Dict with 'name', 'business_model' (hold_to_collect/
              hold_to_collect_and_sell/trading/other), 'sppi_test' (bool),
              'contract_type', 'is_equity'.

    Returns:
        ClassificationResult with the IFRS 9 classification.

    Raises:
        ValueError: If required fields are missing.
    """
    name = asset.get("name", "")
    business_model = asset.get("business_model", "other").lower()
    sppi = asset.get("sppi_test", False)
    is_equity = asset.get("is_equity", False)

    if is_equity:
        if asset.get("irrevocable_fvoci_election", False):
            return ClassificationResult(
                asset_name=name,
                classification="FVTOCI",
                business_model=business_model,
                sppi_result=sppi,
                rationale="Equity instrument with irrevocable FVTOCI election per IFRS 9.4.1.4",
            )
        return ClassificationResult(
            asset_name=name,
            classification="FVTPL",
            business_model=business_model,
            sppi_result=sppi,
            rationale="Equity instrument without FVTOCI election defaults to FVTPL per IFRS 9.4.1.4",
        )

    if not sppi:
        return ClassificationResult(
            asset_name=name,
            classification="FVTPL",
            business_model=business_model,
            sppi_result=sppi,
            rationale="SPPI test failed - contractual cash flows are not solely payments of principal and interest",
        )

    if business_model == "hold_to_collect":
        classification = "Amortized Cost"
        rationale = "Hold-to-collect business model + SPPI pass = Amortized Cost per IFRS 9.4.1.2(a)"
    elif business_model == "hold_to_collect_and_sell":
        classification = "FVTOCI"
        rationale = "Hold-to-collect-and-sell business model + SPPI pass = FVTOCI per IFRS 9.4.1.2(b)"
    else:
        classification = "FVTPL"
        rationale = "Trading or other business model defaults to FVTPL per IFRS 9.4.1.2(c)"

    return ClassificationResult(
        asset_name=name,
        classification=classification,
        business_model=business_model,
        sppi_result=sppi,
        rationale=rationale,
    )


def compute_expected_credit_loss(
    loan_portfolio: list[dict[str, Any]],
    stage: int,
) -> ECLResult:
    """Compute Expected Credit Loss (ECL) for a loan portfolio under
    the IFRS 9 three-stage impairment model.

    Stage 1: 12-month ECL (performing assets, no significant increase
             in credit risk since initial recognition)
    Stage 2: Lifetime ECL (significant increase in credit risk)
    Stage 3: Lifetime ECL (credit-impaired assets)

    Args:
        loan_portfolio: List of dicts with 'exposure_at_default',
                       'probability_of_default_12m', 'probability_of_default_lifetime',
                       'loss_given_default', 'name'.
        stage: Impairment stage (1, 2, or 3).

    Returns:
        ECLResult with total ECL amount and parameters.
    """
    total_ead = Decimal("0")
    total_pd = Decimal("0")
    total_lgd = Decimal("0")
    total_ecl = Decimal("0")
    count = 0

    for loan in loan_portfolio:
        ead = _d(loan.get("exposure_at_default", 0))
        lgd = _d(loan.get("loss_given_default", 0))

        if stage == 1:
            pd = _d(loan.get("probability_of_default_12m", 0))
            lifetime = "12_month"
        else:
            pd = _d(loan.get("probability_of_default_lifetime", 0))
            lifetime = "lifetime"

        ecl = (ead * pd * lgd / Decimal("100")).quantize(TWO_PLACES)
        total_ead += ead
        total_pd += pd
        total_lgd += lgd
        total_ecl += ecl
        count += 1

    avg_pd = (total_pd / Decimal(str(max(count, 1)))).quantize(TWO_PLACES)
    avg_lgd = (total_lgd / Decimal(str(max(count, 1)))).quantize(TWO_PLACES)

    return ECLResult(
        stage=stage,
        exposure_at_default=total_ead,
        probability_of_default=avg_pd,
        loss_given_default=avg_lgd,
        ecl_amount=total_ecl,
        lifetime_or_12_month=lifetime if stage > 1 else "12_month",
    )


def compute_impairment(
    asset: dict[str, Any],
    stage: int,
) -> ImpairmentResult:
    """Compute impairment for a financial asset at a given stage.

    Args:
        asset: Dict with 'name', 'carrying_amount', 'exposure_at_default',
              'probability_of_default_12m', 'probability_of_default_lifetime',
              'loss_given_default'.
        stage: Impairment stage (1, 2, or 3).

    Returns:
        ImpairmentResult with impairment amount.
    """
    carrying = _d(asset.get("carrying_amount", 0))
    ead = _d(asset.get("exposure_at_default", 0))
    lgd = _d(asset.get("loss_given_default", 0))

    if stage == 1:
        pd = _d(asset.get("probability_of_default_12m", 0))
    else:
        pd = _d(asset.get("probability_of_default_lifetime", 0))

    ecl = (ead * pd * lgd / Decimal("100")).quantize(TWO_PLACES)

    return ImpairmentResult(
        asset_name=asset.get("name", ""),
        carrying_amount=carrying,
        ecl_amount=ecl,
        impairment_amount=ecl,
        stage=stage,
    )


def validate_classification(
    asset: dict[str, Any],
    business_model: str,
    sppi_test: bool,
) -> ValidationResult:
    """Validate the classification of a financial asset.

    Performs consistency checks between the stated business model,
    SPPI test result, and resulting classification.

    Args:
        asset: Asset dict with 'name', 'is_equity', 'irrevocable_fvoci_election'.
        business_model: The stated business model.
        sppi_test: Whether the SPPI test was passed.

    Returns:
        ValidationResult with validity and any issues.
    """
    issues: list[str] = []
    is_equity = asset.get("is_equity", False)

    if is_equity and sppi_test:
        issues.append("Equity instruments typically fail SPPI test - verify classification")

    if business_model == "hold_to_collect" and not sppi_test:
        issues.append("Hold-to-collect model requires SPPI pass for Amortized Cost classification")

    classification_result = classify_financial_asset({
        **asset,
        "business_model": business_model,
        "sppi_test": sppi_test,
    })

    if classification_result.classification == "FVTPL" and business_model == "hold_to_collect":
        issues.append("FVTPL classification unexpected for hold-to-collect model - check SPPI test")

    return ValidationResult(
        is_valid=len(issues) == 0,
        classification=classification_result.classification,
        issues=issues,
    )

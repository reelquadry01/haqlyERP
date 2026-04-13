"""IAS 37 Provisions, Contingent Liabilities and Contingent Assets.

Author: Quadri Atharu

Implements IAS 37 provision recognition, measurement, and
classification of contingent liabilities and assets based on
probability thresholds and reliable estimation criteria.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class Provision:
    description: str
    amount: Decimal
    probability: str
    reliable_estimate: bool
    recognition_basis: str


@dataclass
class ContingentLiability:
    description: str
    possible_obligation: bool
    probability: str
    measurement: str
    disclosure_required: bool


@dataclass
class ContingentAssetDisclosure:
    description: str
    virtually_certain: bool
    probable: bool
    disclosure_treatment: str


def recognize_provision(
    obligation: str,
    probability: str,
    reliable_estimate: bool,
) -> Provision:
    """Recognize a provision per IAS 37.14.

    A provision shall be recognized when all three conditions are met:
    1. Present obligation (legal or constructive) from a past event
    2. Probable that outflow of resources will be required to settle
    3. A reliable estimate can be made of the amount

    Args:
        obligation: Description of the obligation.
        probability: 'probable', 'possible', or 'remote'.
        reliable_estimate: Whether a reliable estimate can be made.

    Returns:
        Provision object if recognized, or with zero amount if not.
    """
    if probability == "probable" and reliable_estimate:
        return Provision(
            description=obligation,
            amount=Decimal("0"),
            probability=probability,
            reliable_estimate=reliable_estimate,
            recognition_basis="All three IAS 37.14 conditions met - provision recognized",
        )
    else:
        reasons = []
        if probability != "probable":
            reasons.append(f"outflow not probable ('{probability}')")
        if not reliable_estimate:
            reasons.append("reliable estimate not available")
        return Provision(
            description=obligation,
            amount=Decimal("0"),
            probability=probability,
            reliable_estimate=reliable_estimate,
            recognition_basis=f"Provision not recognized: {'; '.join(reasons)}",
        )


def compute_provision_amount(
    best_estimate: Decimal,
    range_low: Decimal,
    range_high: Decimal,
    probability: Decimal,
) -> Decimal:
    """Compute the provision amount per IAS 37 measurement rules.

    If a single obligation is being measured, the most likely outcome
    is the best estimate. If there is a range of possible outcomes,
    the expected value (weighted average) should be used per IAS 37.39.

    For a continuous range with equal probability, the midpoint is used.

    Args:
        best_estimate: The most likely outcome amount.
        range_low: Lowest estimate in the range.
        range_high: Highest estimate in the range.
        probability: Probability of the best estimate (0-100).

    Returns:
        The provision amount to recognize.
    """
    best = _d(best_estimate)
    low = _d(range_low)
    high = _d(range_high)
    prob = _d(probability)

    if prob >= Decimal("50"):
        return best

    midpoint = ((low + high) / Decimal("2")).quantize(TWO_PLACES)
    weighted = (best * prob / Decimal("100") + midpoint * (Decimal("100") - prob) / Decimal("100")).quantize(TWO_PLACES)

    return weighted


def classify_contingent_liability(
    obligation: str,
    probability: str,
) -> ContingentLiability:
    """Classify a contingent liability per IAS 37.

    - Probable outflow: Recognize as provision
    - Possible outflow: Disclose as contingent liability
    - Remote outflow: No disclosure required

    Args:
        obligation: Description of the possible obligation.
        probability: 'probable', 'possible', or 'remote'.

    Returns:
        ContingentLiability with classification and disclosure requirements.
    """
    if probability == "probable":
        return ContingentLiability(
            description=obligation,
            possible_obligation=False,
            probability=probability,
            measurement="Recognize as provision - outflow probable",
            disclosure_required=True,
        )
    elif probability == "possible":
        return ContingentLiability(
            description=obligation,
            possible_obligation=True,
            probability=probability,
            measurement="Cannot measure reliably or outflow only possible",
            disclosure_required=True,
        )
    else:
        return ContingentLiability(
            description=obligation,
            possible_obligation=True,
            probability=probability,
            measurement="Remote probability - no provision or disclosure",
            disclosure_required=False,
        )


def classify_contingent_asset(
    virtually_certain: bool,
    probable: bool,
) -> ContingentAssetDisclosure:
    """Classify a contingent asset per IAS 37.

    - Virtually certain: Recognize as asset (extremely rare)
    - Probable: Disclose contingent asset in notes
    - Not probable: No disclosure

    Args:
        virtually_certain: Whether inflow is virtually certain.
        probable: Whether inflow is probable (but not virtually certain).

    Returns:
        ContingentAssetDisclosure with disclosure treatment.
    """
    if virtually_certain:
        return ContingentAssetDisclosure(
            description="",
            virtually_certain=True,
            probable=True,
            disclosure_treatment="Virtually certain inflow - recognize as asset per IAS 37.33 (extremely rare case)",
        )
    elif probable:
        return ContingentAssetDisclosure(
            description="",
            virtually_certain=False,
            probable=True,
            disclosure_treatment="Probable inflow - disclose as contingent asset in notes per IAS 37.89 (do not recognize)",
        )
    else:
        return ContingentAssetDisclosure(
            description="",
            virtually_certain=False,
            probable=False,
            disclosure_treatment="Inflow not probable - no disclosure required per IAS 37.90",
        )

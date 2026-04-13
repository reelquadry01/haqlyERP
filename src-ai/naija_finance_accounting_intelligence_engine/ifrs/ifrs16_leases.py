"""IFRS 16 Lease accounting implementation.

Author: Quadri Atharu

Implements IFRS 16 lease classification, right-of-use (ROU) asset
computation, lease liability calculation, depreciation schedules,
and interest expense for lessees. All leases are on-balance-sheet
for lessees under IFRS 16 (short-term and low-value exemptions apply).
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class LeaseClassification:
    lease_id: str
    classification: str
    is_short_term: bool
    is_low_value: bool
    rationale: str


@dataclass
class LeaseLiabilitySchedule:
    period: int
    opening_balance: Decimal
    payment: Decimal
    interest: Decimal
    principal: Decimal
    closing_balance: Decimal


@dataclass
class DepreciationScheduleEntry:
    period: int
    opening_balance: Decimal
    depreciation: Decimal
    closing_balance: Decimal


def classify_lease(lease_data: dict[str, Any]) -> LeaseClassification:
    """Classify a lease for lessee accounting under IFRS 16.

    Under IFRS 16, lessees recognize all leases on-balance-sheet
    (ROU asset + lease liability) unless the lease is:
    - Short-term (<= 12 months) with no purchase option, OR
    - Low-value asset (<= ~USD 5,000 when new)

    For lessors, classification is Operating vs Finance based on
    whether substantially all risks and rewards are transferred.

    Args:
        lease_data: Dict with 'lease_id', 'term_months',
                  'has_purchase_option', 'asset_value_new',
                  'transfer_ownership', 'economic_life_pct',
                  'specialized_asset', 'present_value_pct'.

    Returns:
        LeaseClassification with type and exemption status.
    """
    lease_id = lease_data.get("lease_id", "")
    term_months = int(lease_data.get("term_months", 0))
    has_purchase = lease_data.get("has_purchase_option", False)
    asset_value = _d(lease_data.get("asset_value_new", 0))
    transfer_ownership = lease_data.get("transfer_ownership", False)

    is_short_term = term_months <= 12 and not has_purchase
    is_low_value = asset_value <= Decimal("5000")

    if is_short_term:
        return LeaseClassification(
            lease_id=lease_id,
            classification="short_term_exemption",
            is_short_term=True,
            is_low_value=False,
            rationale="Lease term <= 12 months with no purchase option - short-term exemption per IFRS 16.5",
        )

    if is_low_value:
        return LeaseClassification(
            lease_id=lease_id,
            classification="low_value_exemption",
            is_short_term=False,
            is_low_value=True,
            rationale="Asset is low-value (<= USD 5,000 when new) - low-value exemption per IFRS 16.5A-5B",
        )

    classification = "on_balance_sheet"
    rationale = "Lease recognized on-balance-sheet per IFRS 16 (ROU asset + lease liability)"

    if transfer_ownership:
        rationale = "Ownership transfers - finance lease classification for lessor, on-balance-sheet for lessee"

    return LeaseClassification(
        lease_id=lease_id,
        classification=classification,
        is_short_term=False,
        is_low_value=False,
        rationale=rationale,
    )


def compute_lease_liability(
    payments: list[Decimal],
    discount_rate: Decimal,
    term: int,
) -> list[LeaseLiabilitySchedule]:
    """Compute the lease liability amortization schedule.

    The initial lease liability is the present value of lease payments
    not yet paid, discounted at the rate implicit in the lease or the
    lessee's incremental borrowing rate.

    Args:
        payments: List of lease payments (one per period).
        discount_rate: Discount rate per period (as percentage).
        term: Number of payment periods.

    Returns:
        List of LeaseLiabilitySchedule entries showing opening balance,
        payment, interest, principal, and closing balance per period.
    """
    rate = _d(discount_rate) / Decimal("100")
    schedule: list[LeaseLiabilitySchedule] = []

    pv = Decimal("0")
    for i, payment in enumerate(payments[:term]):
        pv += _d(payment) / ((Decimal("1") + rate) ** Decimal(str(i + 1)))
    pv = pv.quantize(TWO_PLACES)

    balance = pv

    for period_num in range(min(term, len(payments))):
        payment = _d(payments[period_num])
        interest = (balance * rate).quantize(TWO_PLACES)
        principal = (payment - interest).quantize(TWO_PLACES)
        closing = (balance - principal).quantize(TWO_PLACES)

        if closing < Decimal("0"):
            closing = Decimal("0")

        schedule.append(LeaseLiabilitySchedule(
            period=period_num + 1,
            opening_balance=balance,
            payment=payment,
            interest=interest,
            principal=principal,
            closing_balance=closing,
        ))

        balance = closing

    return schedule


def compute_rou_asset(
    liability: Decimal,
    initial_direct_costs: Decimal,
    prepayments: Decimal,
) -> Decimal:
    """Compute the right-of-use asset at initial recognition.

    Per IFRS 16.24, the ROU asset comprises:
    - Initial measurement of lease liability
    + Lease payments made at or before commencement date
    + Initial direct costs incurred by lessee
    - Lease incentives received

    Args:
        liability: Initial lease liability amount.
        initial_direct_costs: Initial direct costs of obtaining the lease.
        prepayments: Lease payments made before commencement date.

    Returns:
        ROU asset amount at initial recognition.
    """
    liability = _d(liability)
    initial_direct_costs = _d(initial_direct_costs)
    prepayments = _d(prepayments)

    return (liability + initial_direct_costs + prepayments).quantize(TWO_PLACES)


def compute_lease_depreciation(
    rou_asset: Decimal,
    term: int,
) -> list[DepreciationScheduleEntry]:
    """Compute the ROU asset depreciation schedule using straight-line method.

    Per IFRS 16.31, the ROU asset is depreciated over the shorter of:
    - The useful life of the underlying asset, OR
    - The lease term

    If ownership transfers or purchase option is reasonably certain,
    depreciate over the asset's useful life instead.

    Args:
        rou_asset: Initial ROU asset amount.
        term: Number of periods over which to depreciate.

    Returns:
        List of DepreciationScheduleEntry objects.
    """
    rou_asset = _d(rou_asset)
    term = max(term, 1)

    annual_dep = (rou_asset / Decimal(str(term))).quantize(TWO_PLACES)
    schedule: list[DepreciationScheduleEntry] = []
    balance = rou_asset

    for period_num in range(term):
        if period_num == term - 1:
            dep = balance
        else:
            dep = annual_dep

        closing = (balance - dep).quantize(TWO_PLACES)
        if closing < Decimal("0"):
            closing = Decimal("0")
            dep = balance

        schedule.append(DepreciationScheduleEntry(
            period=period_num + 1,
            opening_balance=balance,
            depreciation=dep,
            closing_balance=closing,
        ))
        balance = closing

    return schedule


def compute_lease_interest(
    liability_balance: Decimal,
    discount_rate: Decimal,
) -> Decimal:
    """Compute interest expense on the lease liability for a period.

    Per IFRS 16.36, interest on the lease liability produces a constant
    periodic rate of interest on the outstanding liability.

    Args:
        liability_balance: Opening lease liability balance.
        discount_rate: Periodic discount rate as percentage.

    Returns:
        Interest expense amount for the period.
    """
    liability_balance = _d(liability_balance)
    discount_rate = _d(discount_rate)

    return (liability_balance * discount_rate / Decimal("100")).quantize(TWO_PLACES)

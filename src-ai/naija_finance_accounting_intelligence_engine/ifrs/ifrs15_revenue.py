"""IFRS 15 Revenue from Contracts with Customers - five-step model.

Author: Quadri Atharu

Implements the IFRS 15 five-step revenue recognition framework:
1. Identify the contract
2. Identify performance obligations
3. Determine the transaction price
4. Allocate the transaction price
5. Recognize revenue when obligations are satisfied

Full orchestration via apply_five_step_model().
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class Contract:
    contract_id: str
    customer: str
    start_date: str
    end_date: str
    is_approved: bool
    has_identifiable_rights: bool
    payment_terms: str
    has_commercial_substance: bool
    collectibility_probable: bool


@dataclass
class PerformanceObligation:
    obligation_id: str
    description: str
    stand_alone_price: Decimal
    is_distinct: bool
    satisfaction_method: str  # "over_time" or "point_in_time"


@dataclass
class PriceAllocation:
    obligation_id: str
    allocated_amount: Decimal
    stand_alone_price: Decimal
    allocation_pct: Decimal


@dataclass
class RevenueEntry:
    obligation_id: str
    amount: Decimal
    date: str
    debit_account: str
    credit_account: str
    satisfaction_method: str


@dataclass
class RevenueRecognition:
    contract_id: str
    total_transaction_price: Decimal
    obligations: list[PerformanceObligation]
    allocations: list[PriceAllocation]
    revenue_entries: list[RevenueEntry]
    deferred_revenue: Decimal = Decimal("0")


def identify_contract(contract_data: dict[str, Any]) -> Contract:
    """Step 1: Identify the contract with a customer.

    Per IFRS 15.9, a contract exists if:
    - Parties have approved and committed to the contract
    - Rights regarding goods/services are identifiable
    - Payment terms are identifiable
    - Contract has commercial substance
    - Collectibility of consideration is probable

    Args:
        contract_data: Dict with 'contract_id', 'customer', 'start_date',
                      'end_date', 'is_approved', 'has_identifiable_rights',
                      'payment_terms', 'has_commercial_substance',
                      'collectibility_probable'.

    Returns:
        Contract object if all criteria are met.

    Raises:
        ValueError: If contract identification criteria are not met.
    """
    is_approved = contract_data.get("is_approved", False)
    has_rights = contract_data.get("has_identifiable_rights", False)
    has_substance = contract_data.get("has_commercial_substance", False)
    collectible = contract_data.get("collectibility_probable", False)

    if not all([is_approved, has_rights, has_substance, collectible]):
        missing = []
        if not is_approved:
            missing.append("contract not approved by parties")
        if not has_rights:
            missing.append("identifiable rights not established")
        if not has_substance:
            missing.append("no commercial substance")
        if not collectible:
            missing.append("collectibility not probable")
        raise ValueError(f"Contract criteria not met: {'; '.join(missing)}")

    return Contract(
        contract_id=contract_data.get("contract_id", ""),
        customer=contract_data.get("customer", ""),
        start_date=contract_data.get("start_date", ""),
        end_date=contract_data.get("end_date", ""),
        is_approved=is_approved,
        has_identifiable_rights=has_rights,
        payment_terms=contract_data.get("payment_terms", ""),
        has_commercial_substance=has_substance,
        collectibility_probable=collectible,
    )


def identify_performance_obligations(
    contract: Contract,
    obligations_data: list[dict[str, Any]],
) -> list[PerformanceObligation]:
    """Step 2: Identify distinct performance obligations.

    A good/service is distinct if:
    - Customer can benefit from it on its own (capable of being distinct)
    - It is separately identifiable from other promises (distinct in context)

    Args:
        contract: The identified Contract.
        obligations_data: List of dicts with 'obligation_id', 'description',
                         'stand_alone_price', 'is_distinct', 'satisfaction_method'.

    Returns:
        List of PerformanceObligation for distinct obligations only.
    """
    obligations: list[PerformanceObligation] = []

    for obl in obligations_data:
        if obl.get("is_distinct", True):
            obligations.append(PerformanceObligation(
                obligation_id=obl.get("obligation_id", ""),
                description=obl.get("description", ""),
                stand_alone_price=_d(obl.get("stand_alone_price", 0)),
                is_distinct=True,
                satisfaction_method=obl.get("satisfaction_method", "point_in_time"),
            ))

    return obligations


def determine_transaction_price(
    contract: Contract,
    obligations: list[PerformanceObligation],
) -> Decimal:
    """Step 3: Determine the transaction price.

    The transaction price is the amount of consideration the entity
    expects to be entitled to, including variable consideration
    (constrained), significant financing component, non-cash
    consideration, and consideration payable to the customer.

    Args:
        contract: The identified Contract.
        obligations: List of PerformanceObligation.

    Returns:
        Total transaction price as Decimal.
    """
    total_stand_alone = sum(o.stand_alone_price for o in obligations)
    return total_stand_alone.quantize(TWO_PLACES)


def allocate_transaction_price(
    obligations: list[PerformanceObligation],
    price: Decimal,
) -> list[PriceAllocation]:
    """Step 4: Allocate the transaction price to performance obligations.

    Based on relative stand-alone selling prices. If stand-alone prices
    are not directly observable, use adjusted market assessment approach
    or expected cost plus margin approach.

    Args:
        obligations: List of PerformanceObligation.
        price: Total transaction price.

    Returns:
        List of PriceAllocation with allocated amounts per obligation.
    """
    total_stand_alone = sum(o.stand_alone_price for o in obligations)

    allocations: list[PriceAllocation] = []
    if total_stand_alone == Decimal("0"):
        per_obligation = price / Decimal(str(max(len(obligations), 1)))
        for obl in obligations:
            allocations.append(PriceAllocation(
                obligation_id=obl.obligation_id,
                allocated_amount=per_obligation.quantize(TWO_PLACES),
                stand_alone_price=Decimal("0"),
                allocation_pct=Decimal(str(round(100 / max(len(obligations), 1), 2))),
            ))
    else:
        for obl in obligations:
            alloc_pct = (obl.stand_alone_price / total_stand_alone * Decimal("100")).quantize(TWO_PLACES)
            allocated = (price * obl.stand_alone_price / total_stand_alone).quantize(TWO_PLACES)
            allocations.append(PriceAllocation(
                obligation_id=obl.obligation_id,
                allocated_amount=allocated,
                stand_alone_price=obl.stand_alone_price,
                allocation_pct=alloc_pct,
            ))

    return allocations


def recognize_revenue(
    obligation: PerformanceObligation,
    satisfaction_point: str,
) -> RevenueEntry:
    """Step 5: Recognize revenue when a performance obligation is satisfied.

    Revenue is recognized when (or as) the entity transfers control
    of a good/service to the customer.

    Args:
        obligation: The satisfied PerformanceObligation.
        satisfaction_point: Date or description of satisfaction.

    Returns:
        RevenueEntry with the recognition details.
    """
    return RevenueEntry(
        obligation_id=obligation.obligation_id,
        amount=obligation.stand_alone_price,
        date=satisfaction_point,
        debit_account="Contract Asset / Receivables",
        credit_account="Revenue from Contracts with Customers",
        satisfaction_method=obligation.satisfaction_method,
    )


def apply_five_step_model(
    contract_data: dict[str, Any],
) -> RevenueRecognition:
    """Apply the complete IFRS 15 five-step revenue recognition model.

    Orchestrates all five steps from contract identification through
    revenue recognition.

    Args:
        contract_data: Dict with contract fields plus 'obligations'
                      (list of obligation dicts) and 'variable_consideration'.

    Returns:
        RevenueRecognition with complete five-step results.
    """
    contract = identify_contract(contract_data)

    obligations = identify_performance_obligations(
        contract,
        contract_data.get("obligations", []),
    )

    if not obligations:
        raise ValueError("No distinct performance obligations identified")

    transaction_price = determine_transaction_price(contract, obligations)

    variable = _d(contract_data.get("variable_consideration", 0))
    transaction_price += variable

    allocations = allocate_transaction_price(obligations, transaction_price)

    revenue_entries: list[RevenueEntry] = []
    deferred = Decimal("0")

    for alloc in allocations:
        matching_obl = next(
            (o for o in obligations if o.obligation_id == alloc.obligation_id),
            None,
        )
        if matching_obl is None:
            continue

        matching_obl.stand_alone_price = alloc.allocated_amount

        if matching_obl.satisfaction_method == "point_in_time":
            entry = recognize_revenue(matching_obl, contract.end_date)
            entry.amount = alloc.allocated_amount
            revenue_entries.append(entry)
        else:
            entry = recognize_revenue(matching_obl, contract.start_date)
            entry.amount = alloc.allocated_amount
            revenue_entries.append(entry)

    recognized_total = sum(e.amount for e in revenue_entries)
    deferred = (transaction_price - recognized_total).quantize(TWO_PLACES)

    return RevenueRecognition(
        contract_id=contract.contract_id,
        total_transaction_price=transaction_price,
        obligations=obligations,
        allocations=allocations,
        revenue_entries=revenue_entries,
        deferred_revenue=deferred,
    )

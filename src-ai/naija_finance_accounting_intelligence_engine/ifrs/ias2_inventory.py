"""IAS 2 Inventories - cost computation and NRV valuation.

Author: Quadri Atharu

Implements IAS 2 inventory cost methods (FIFO, Weighted Average),
net realisable value computation, and lower-of-cost-or-NRV
valuation with write-down recognition.
"""

from __future__ import annotations

from dataclasses import dataclass
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class InventoryCostResult:
    method: str
    items: list[dict[str, Decimal]]
    total_cost: Decimal


@dataclass
class NRVResult:
    selling_price: Decimal
    costs_to_sell: Decimal
    nrv: Decimal


@dataclass
class WriteDownEntry:
    item: str
    cost: Decimal
    nrv: Decimal
    write_down_amount: Decimal
    debit_account: str
    credit_account: str


def compute_inventory_cost(
    items: list[dict[str, Any]],
    method: str,
) -> InventoryCostResult:
    """Compute inventory cost using FIFO or Weighted Average method.

    Per IAS 2, acceptable cost formulas are FIFO and Weighted Average.
    LIFO is NOT permitted under IAS 2.

    Args:
        items: List of dicts with 'name', 'quantity', 'unit_cost',
              'purchase_order' (for FIFO ordering).
        method: 'FIFO' or 'WeightedAverage'.

    Returns:
        InventoryCostResult with computed costs per item and total.

    Raises:
        ValueError: If method is not supported.
    """
    method = method.upper()
    if method not in ("FIFO", "WEIGHTEDAVERAGE", "WAVG"):
        raise ValueError(f"Method '{method}' not supported. Use 'FIFO' or 'WeightedAverage'")

    computed_items: list[dict[str, Decimal]] = []
    total_cost = Decimal("0")

    if method in ("WEIGHTEDAVERAGE", "WAVG"):
        total_quantity = sum(_d(item.get("quantity", 0)) for item in items)
        total_value = sum(_d(item.get("quantity", 0)) * _d(item.get("unit_cost", 0)) for item in items)

        if total_quantity > Decimal("0"):
            avg_cost = (total_value / total_quantity).quantize(TWO_PLACES)
        else:
            avg_cost = Decimal("0")

        for item in items:
            qty = _d(item.get("quantity", 0))
            cost = (qty * avg_cost).quantize(TWO_PLACES)
            computed_items.append({
                "name": item.get("name", ""),
                "quantity": qty,
                "unit_cost": avg_cost,
                "total_cost": cost,
            })
            total_cost += cost
    else:
        sorted_items = sorted(items, key=lambda x: x.get("purchase_order", 0))
        for item in sorted_items:
            qty = _d(item.get("quantity", 0))
            unit_cost = _d(item.get("unit_cost", 0))
            cost = (qty * unit_cost).quantize(TWO_PLACES)
            computed_items.append({
                "name": item.get("name", ""),
                "quantity": qty,
                "unit_cost": unit_cost,
                "total_cost": cost,
            })
            total_cost += cost

    return InventoryCostResult(
        method=method,
        items=computed_items,
        total_cost=total_cost.quantize(TWO_PLACES),
    )


def compute_nrv(
    inventory: dict[str, Any],
    selling_price: Decimal,
    costs_to_sell: Decimal,
) -> NRVResult:
    """Compute Net Realisable Value per IAS 2.

    NRV = Estimated selling price - Estimated costs of completion
          - Estimated costs necessary to make the sale.

    Args:
        inventory: Inventory item dict (for reference).
        selling_price: Estimated selling price in normal course of business.
        costs_to_sell: Estimated costs to complete and sell.

    Returns:
        NRVResult with computed net realisable value.
    """
    selling_price = _d(selling_price)
    costs_to_sell = _d(costs_to_sell)
    nrv = (selling_price - costs_to_sell).quantize(TWO_PLACES)

    return NRVResult(
        selling_price=selling_price,
        costs_to_sell=costs_to_sell,
        nrv=nrv,
    )


def apply_lower_of_cost_or_nrv(cost: Decimal, nrv: Decimal) -> Decimal:
    """Apply the lower of cost or NRV rule per IAS 2.9.

    Inventories shall be measured at the lower of cost and
    net realisable value.

    Args:
        cost: The computed inventory cost.
        nrv: The computed net realisable value.

    Returns:
        The lower of cost and NRV.
    """
    cost = _d(cost)
    nrv = _d(nrv)
    return min(cost, nrv)


def recognize_inventory_write_down(
    cost: Decimal,
    nrv: Decimal,
) -> WriteDownEntry:
    """Recognize an inventory write-down when NRV is below cost.

    Per IAS 2.28, the write-down should be recognized as an expense
    in the period the write-down occurs. The amount is the difference
    between cost and NRV.

    Args:
        cost: The inventory carrying amount (cost).
        nrv: The net realisable value.

    Returns:
        WriteDownEntry with the write-down amount and accounts.

    Raises:
        ValueError: If cost <= NRV (no write-down needed).
    """
    cost = _d(cost)
    nrv = _d(nrv)

    if cost <= nrv:
        return WriteDownEntry(
            item="",
            cost=cost,
            nrv=nrv,
            write_down_amount=Decimal("0"),
            debit_account="",
            credit_account="",
        )

    write_down = (cost - nrv).quantize(TWO_PLACES)

    return WriteDownEntry(
        item="",
        cost=cost,
        nrv=nrv,
        write_down_amount=write_down,
        debit_account="Inventory Write-down Expense (COS)",
        credit_account="Inventory (Allowance for NRV)",
    )

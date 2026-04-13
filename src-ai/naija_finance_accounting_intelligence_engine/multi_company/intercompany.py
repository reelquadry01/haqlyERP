"""Intercompany transaction elimination for consolidated reporting.

Author: Quadri Atharu

Identifies and eliminates intercompany transactions including sales revenue,
receivables/payables, inventory profits, and loan interest to produce
clean consolidated financial statements per IFRS 10 requirements.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class IntercompanyTransaction:
    from_entity: str
    to_entity: str
    transaction_type: str
    amount: Decimal
    description: str = ""
    date: str = ""
    reference: str = ""
    profit_margin: Decimal = Decimal("0")
    unsold_pct: Decimal = Decimal("0")
    interest_rate: Decimal = Decimal("0")


@dataclass
class EliminationEntry:
    entry_type: str
    description: str
    debit_account: str
    credit_account: str
    amount: Decimal
    from_entity: str
    to_entity: str


def identify_intercompany_transactions(
    parent: dict[str, Any],
    subsidiaries: list[dict[str, Any]],
) -> list[IntercompanyTransaction]:
    """Identify intercompany transactions between parent and subsidiaries
    and among subsidiaries themselves.

    Scans each entity's transaction records for counterparties within
    the group and classifies them by transaction type.

    Args:
        parent: Dict with 'name' and 'transactions' (list of dicts with
                'counterparty', 'type', 'amount', 'date', 'reference',
                optional 'profit_margin', 'unsold_pct', 'interest_rate').
        subsidiaries: List of dicts, same structure as parent.

    Returns:
        List of IntercompanyTransaction objects representing all identified
        intercompany dealings.
    """
    all_entities = {parent["name"]}
    for sub in subsidiaries:
        all_entities.add(sub["name"])

    ic_transactions: list[IntercompanyTransaction] = []
    all_parties = [parent] + subsidiaries

    for entity in all_parties:
        entity_name = entity["name"]
        for txn in entity.get("transactions", []):
            counterparty = txn.get("counterparty", "")
            if counterparty in all_entities and counterparty != entity_name:
                ic_transactions.append(IntercompanyTransaction(
                    from_entity=entity_name,
                    to_entity=counterparty,
                    transaction_type=txn.get("type", "sales"),
                    amount=_d(txn.get("amount", 0)),
                    description=txn.get("description", ""),
                    date=txn.get("date", ""),
                    reference=txn.get("reference", ""),
                    profit_margin=_d(txn.get("profit_margin", 0)),
                    unsold_pct=_d(txn.get("unsold_pct", 0)),
                    interest_rate=_d(txn.get("interest_rate", 0)),
                ))

    return ic_transactions


def eliminate_ic_sales_revenue(
    ic_transactions: list[IntercompanyTransaction],
) -> list[EliminationEntry]:
    """Eliminate intercompany sales revenue and corresponding cost of sales.

    Removes the double-counting that occurs when one group entity sells
    to another, ensuring consolidated revenue and cost of sales reflect
    only external transactions.

    Args:
        ic_transactions: List of IntercompanyTransaction objects, filtered
                        to those with transaction_type 'sales'.

    Returns:
        List of EliminationEntry objects reversing IC sales and COGS.
    """
    elimination_entries: list[EliminationEntry] = []

    sales_txns = [t for t in ic_transactions if t.transaction_type == "sales"]

    for txn in sales_txns:
        elimination_entries.append(EliminationEntry(
            entry_type="eliminate_ic_revenue",
            description=f"Eliminate IC sales revenue: {txn.from_entity} -> {txn.to_entity}",
            debit_account="Revenue (Intercompany)",
            credit_account="Cost of Sales (Intercompany)",
            amount=txn.amount,
            from_entity=txn.from_entity,
            to_entity=txn.to_entity,
        ))

    return elimination_entries


def eliminate_ic_receivables_payables(
    ic_transactions: list[IntercompanyTransaction],
) -> list[EliminationEntry]:
    """Eliminate intercompany receivables and payables.

    Removes matching intercompany receivable/payable balances so that
    the consolidated balance sheet shows only external obligations.

    Args:
        ic_transactions: List of IntercompanyTransaction objects, filtered
                        to those with transaction_type 'receivable'.

    Returns:
        List of EliminationEntry objects offsetting IC receivables/payables.
    """
    elimination_entries: list[EliminationEntry] = []

    receivable_txns = [t for t in ic_transactions if t.transaction_type == "receivable"]

    net_positions: dict[tuple[str, str], Decimal] = {}
    for txn in receivable_txns:
        pair = (txn.from_entity, txn.to_entity)
        net_positions[pair] = net_positions.get(pair, Decimal("0")) + txn.amount

    for (from_ent, to_ent), net_amount in net_positions.items():
        if net_amount > Decimal("0"):
            elimination_entries.append(EliminationEntry(
                entry_type="eliminate_ic_receivable_payable",
                description=f"Eliminate IC receivable/payable: {from_ent} <-> {to_ent}",
                debit_account="Intercompany Payable",
                credit_account="Intercompany Receivable",
                amount=net_amount,
                from_entity=from_ent,
                to_entity=to_ent,
            ))

    return elimination_entries


def eliminate_ic_inventory_profit(
    ic_transactions: list[IntercompanyTransaction],
) -> list[EliminationEntry]:
    """Eliminate unrealized profit in intercompany inventory.

    When goods sold intercompany remain in the buyer's closing inventory,
    the profit element must be eliminated from consolidated inventory
    and cost of sales. This implements the downstream/upstream profit
    elimination per IFRS 10.B86.

    Args:
        ic_transactions: List of IntercompanyTransaction objects, filtered
                        to those with transaction_type 'inventory_profit'.

    Returns:
        List of EliminationEntry objects removing unrealized IC inventory profit.
    """
    elimination_entries: list[EliminationEntry] = []

    inventory_txns = [t for t in ic_transactions if t.transaction_type == "inventory_profit"]

    for txn in inventory_txns:
        unrealized_profit = txn.amount * txn.profit_margin / Decimal("100") * txn.unsold_pct / Decimal("100")
        if unrealized_profit > Decimal("0"):
            elimination_entries.append(EliminationEntry(
                entry_type="eliminate_ic_inventory_profit",
                description=(
                    f"Eliminate unrealized IC inventory profit: "
                    f"{txn.from_entity} -> {txn.to_entity} "
                    f"(margin={txn.profit_margin}%, unsold={txn.unsold_pct}%)"
                ),
                debit_account="Cost of Sales",
                credit_account="Inventory",
                amount=unrealized_profit.quantize(TWO_PLACES),
                from_entity=txn.from_entity,
                to_entity=txn.to_entity,
            ))

    return elimination_entries


def eliminate_ic_loans_interest(
    ic_transactions: list[IntercompanyTransaction],
) -> list[EliminationEntry]:
    """Eliminate intercompany loan balances and interest income/expense.

    Removes intercompany loan principal balances from the consolidated
    balance sheet and eliminates matching interest income and expense
    from the consolidated income statement.

    Args:
        ic_transactions: List of IntercompanyTransaction objects, filtered
                        to those with transaction_type 'loan' or 'interest'.

    Returns:
        List of EliminationEntry objects eliminating IC loans and interest.
    """
    elimination_entries: list[EliminationEntry] = []

    loan_txns = [t for t in ic_transactions if t.transaction_type == "loan"]
    interest_txns = [t for t in ic_transactions if t.transaction_type == "interest"]

    for txn in loan_txns:
        elimination_entries.append(EliminationEntry(
            entry_type="eliminate_ic_loan",
            description=f"Eliminate IC loan principal: {txn.from_entity} -> {txn.to_entity}",
            debit_account="Intercompany Loan Payable",
            credit_account="Intercompany Loan Receivable",
            amount=txn.amount,
            from_entity=txn.from_entity,
            to_entity=txn.to_entity,
        ))

    for txn in interest_txns:
        elimination_entries.append(EliminationEntry(
            entry_type="eliminate_ic_interest",
            description=f"Eliminate IC interest: {txn.from_entity} -> {txn.to_entity}",
            debit_account="Interest Income (Intercompany)",
            credit_account="Interest Expense (Intercompany)",
            amount=txn.amount,
            from_entity=txn.from_entity,
            to_entity=txn.to_entity,
        ))

    return elimination_entries

"""Consolidated financial statement generation for multi-company groups.

Author: Quadri Atharu

Generates consolidated balance sheets and income statements, adjusts
for parent investments in subsidiaries, and produces elimination
entries for intercompany transactions in accordance with IFRS 10
and Nigerian GAAP requirements.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class BalanceSheetLineItem:
    name: str
    amount: Decimal = Decimal("0")

    def add(self, other: "BalanceSheetLineItem") -> "BalanceSheetLineItem":
        return BalanceSheetLineItem(name=self.name, amount=self.amount + other.amount)


@dataclass
class ConsolidatedBalanceSheet:
    parent_name: str
    period: str
    assets: dict[str, Decimal] = field(default_factory=dict)
    liabilities: dict[str, Decimal] = field(default_factory=dict)
    equity: dict[str, Decimal] = field(default_factory=dict)
    minority_interest: Decimal = Decimal("0")
    elimination_entries: list[dict[str, Any]] = field(default_factory=list)

    def total_assets(self) -> Decimal:
        return sum(self.assets.values(), Decimal("0"))

    def total_liabilities(self) -> Decimal:
        return sum(self.liabilities.values(), Decimal("0"))

    def total_equity(self) -> Decimal:
        return sum(self.equity.values(), Decimal("0"))

    def is_balanced(self) -> bool:
        return self.total_assets() == self.total_liabilities() + self.total_equity() + self.minority_interest


@dataclass
class ConsolidatedIncomeStatement:
    parent_name: str
    period: str
    revenue: Decimal = Decimal("0")
    cost_of_sales: Decimal = Decimal("0")
    gross_profit: Decimal = Decimal("0")
    operating_expenses: Decimal = Decimal("0")
    operating_income: Decimal = Decimal("0")
    other_income: Decimal = Decimal("0")
    finance_costs: Decimal = Decimal("0")
    profit_before_tax: Decimal = Decimal("0")
    tax_expense: Decimal = Decimal("0")
    profit_for_period: Decimal = Decimal("0")
    nci_share: Decimal = Decimal("0")
    parent_share: Decimal = Decimal("0")
    elimination_entries: list[dict[str, Any]] = field(default_factory=list)


@dataclass
class AdjustmentEntry:
    description: str
    debit_account: str
    credit_account: str
    amount: Decimal
    entity: str


def generate_consolidated_balance_sheet(
    parent: dict[str, Any],
    subsidiaries: list[dict[str, Any]],
    period: str,
) -> ConsolidatedBalanceSheet:
    """Generate a consolidated balance sheet by combining parent and subsidiary
    financial positions, then applying investment adjustments and intercompany
    eliminations.

    Args:
        parent: Dict with keys 'name', 'assets', 'liabilities', 'equity',
                'investments_in_subsidiaries'.
        subsidiaries: List of dicts, each with keys 'name', 'assets',
                     'liabilities', 'equity', 'ownership_pct', 'nci_pct'.
        period: Reporting period string (e.g. '2025-12-31').

    Returns:
        ConsolidatedBalanceSheet with combined, adjusted figures.
    """
    consolidated = ConsolidatedBalanceSheet(
        parent_name=parent["name"],
        period=period,
    )

    for key, value in parent.get("assets", {}).items():
        consolidated.assets[key] = consolidated.assets.get(key, Decimal("0")) + _d(value)

    for key, value in parent.get("liabilities", {}).items():
        consolidated.liabilities[key] = consolidated.liabilities.get(key, Decimal("0")) + _d(value)

    for key, value in parent.get("equity", {}).items():
        consolidated.equity[key] = consolidated.equity.get(key, Decimal("0")) + _d(value)

    for sub in subsidiaries:
        ownership_pct = _d(sub.get("ownership_pct", 100))
        nci_pct = Decimal("100") - ownership_pct

        for key, value in sub.get("assets", {}).items():
            consolidated.assets[key] = consolidated.assets.get(key, Decimal("0")) + _d(value)

        for key, value in sub.get("liabilities", {}).items():
            consolidated.liabilities[key] = consolidated.liabilities.get(key, Decimal("0")) + _d(value)

        sub_equity_total = sum(_d(v) for v in sub.get("equity", {}).values())
        parent_equity_share = sub_equity_total * ownership_pct / Decimal("100")
        nci_equity_share = sub_equity_total * nci_pct / Decimal("100")

        for key, value in sub.get("equity", {}).items():
            parent_portion = _d(value) * ownership_pct / Decimal("100")
            consolidated.equity[key] = consolidated.equity.get(key, Decimal("0")) + parent_portion

        consolidated.minority_interest += nci_equity_share

        investment_elimination = AdjustmentEntry(
            description=f"Elimination of investment in {sub['name']}",
            debit_account="Investment in Subsidiary",
            credit_account="Share Capital / Retained Earnings",
            amount=parent_equity_share,
            entity=sub["name"],
        )
        consolidated.elimination_entries.append({
            "type": "investment_elimination",
            "description": investment_elimination.description,
            "debit_account": investment_elimination.debit_account,
            "credit_account": investment_elimination.credit_account,
            "amount": str(investment_elimination.amount),
            "entity": investment_elimination.entity,
        })

        if "Investment in Subsidiary" in consolidated.assets:
            consolidated.assets["Investment in Subsidiary"] -= parent_equity_share
            if consolidated.assets["Investment in Subsidiary"] <= Decimal("0"):
                del consolidated.assets["Investment in Subsidiary"]

    return consolidated


def generate_consolidated_income_statement(
    parent: dict[str, Any],
    subsidiaries: list[dict[str, Any]],
    period: str,
) -> ConsolidatedIncomeStatement:
    """Generate a consolidated income statement by aggregating parent and
    subsidiary results and allocating profit between parent equity holders
    and non-controlling interests.

    Args:
        parent: Dict with keys 'name', 'revenue', 'cost_of_sales',
                'operating_expenses', 'other_income', 'finance_costs',
                'tax_expense'.
        subsidiaries: List of dicts, each with keys 'name', 'revenue',
                     'cost_of_sales', 'operating_expenses', 'other_income',
                     'finance_costs', 'tax_expense', 'ownership_pct'.
        period: Reporting period string.

    Returns:
        ConsolidatedIncomeStatement with combined results and NCI allocation.
    """
    revenue = _d(parent.get("revenue", 0))
    cost_of_sales = _d(parent.get("cost_of_sales", 0))
    operating_expenses = _d(parent.get("operating_expenses", 0))
    other_income = _d(parent.get("other_income", 0))
    finance_costs = _d(parent.get("finance_costs", 0))
    tax_expense = _d(parent.get("tax_expense", 0))

    total_nci_share = Decimal("0")

    for sub in subsidiaries:
        revenue += _d(sub.get("revenue", 0))
        cost_of_sales += _d(sub.get("cost_of_sales", 0))
        operating_expenses += _d(sub.get("operating_expenses", 0))
        other_income += _d(sub.get("other_income", 0))
        finance_costs += _d(sub.get("finance_costs", 0))
        tax_expense += _d(sub.get("tax_expense", 0))

    gross_profit = revenue - cost_of_sales
    operating_income = gross_profit - operating_expenses
    profit_before_tax = operating_income + other_income - finance_costs
    profit_for_period = profit_before_tax - tax_expense

    for sub in subsidiaries:
        ownership_pct = _d(sub.get("ownership_pct", 100))
        nci_pct = Decimal("100") - ownership_pct
        sub_profit = (
            _d(sub.get("revenue", 0))
            - _d(sub.get("cost_of_sales", 0))
            - _d(sub.get("operating_expenses", 0))
            + _d(sub.get("other_income", 0))
            - _d(sub.get("finance_costs", 0))
            - _d(sub.get("tax_expense", 0))
        )
        total_nci_share += sub_profit * nci_pct / Decimal("100")

    parent_share = profit_for_period - total_nci_share

    return ConsolidatedIncomeStatement(
        parent_name=parent["name"],
        period=period,
        revenue=revenue,
        cost_of_sales=cost_of_sales,
        gross_profit=gross_profit,
        operating_expenses=operating_expenses,
        operating_income=operating_income,
        other_income=other_income,
        finance_costs=finance_costs,
        profit_before_tax=profit_before_tax,
        tax_expense=tax_expense,
        profit_for_period=profit_for_period,
        nci_share=total_nci_share,
        parent_share=parent_share,
    )


def adjust_for_investments(
    parent: dict[str, Any],
    subsidiary: dict[str, Any],
    equity_percentage: Decimal,
) -> list[AdjustmentEntry]:
    """Create adjustment entries for parent's investment in a subsidiary.

    Under IFRS 10, the parent's investment carrying amount is eliminated
    against the parent's share of the subsidiary's equity. Goodwill or
    bargain purchase gain arises on consolidation.

    Args:
        parent: Dict with 'name' and 'investment_in_subsidiary' amount.
        subsidiary: Dict with 'name' and 'equity' dict of equity components.
        equity_percentage: Parent's ownership percentage (0-100).

    Returns:
        List of AdjustmentEntry objects representing the consolidation
        adjustments.
    """
    adjustments: list[AdjustmentEntry] = []
    investment_amount = _d(parent.get("investment_in_subsidiary", 0))
    equity_percentage = _d(equity_percentage)
    sub_equity_total = sum(_d(v) for v in subsidiary.get("equity", {}).values())
    parent_share_of_equity = sub_equity_total * equity_percentage / Decimal("100")

    adjustments.append(AdjustmentEntry(
        description=f"Eliminate investment in {subsidiary['name']} against equity",
        debit_account="Share Capital & Reserves",
        credit_account="Investment in Subsidiary",
        amount=parent_share_of_equity,
        entity=subsidiary["name"],
    ))

    goodwill = investment_amount - parent_share_of_equity
    if goodwill > Decimal("0"):
        adjustments.append(AdjustmentEntry(
            description=f"Recognize goodwill on acquisition of {subsidiary['name']}",
            debit_account="Goodwill",
            credit_account="Investment in Subsidiary",
            amount=goodwill,
            entity=subsidiary["name"],
        ))
    elif goodwill < Decimal("0"):
        adjustments.append(AdjustmentEntry(
            description=f"Recognize bargain purchase gain on {subsidiary['name']}",
            debit_account="Investment in Subsidiary",
            credit_account="Gain on Bargain Purchase",
            amount=abs(goodwill),
            entity=subsidiary["name"],
        ))

    nci_pct = Decimal("100") - equity_percentage
    if nci_pct > Decimal("0"):
        nci_equity = sub_equity_total * nci_pct / Decimal("100")
        adjustments.append(AdjustmentEntry(
            description=f"Recognize non-controlling interest in {subsidiary['name']}",
            debit_account="Share Capital & Reserves",
            credit_account="Non-Controlling Interest",
            amount=nci_equity,
            entity=subsidiary["name"],
        ))

    return adjustments


def eliminate_intercompany(
    transactions: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    """Generate elimination entries for intercompany transactions.

    Processes each intercompany transaction and creates corresponding
    elimination journal entries that remove double-counting in the
    consolidated financial statements.

    Args:
        transactions: List of dicts with keys 'type' (sales/receivable/
                     loan/inventory_profit), 'from_entity', 'to_entity',
                     'amount', and optional 'profit_margin', 'unsold_pct',
                     'interest_rate'.

    Returns:
        List of elimination entry dicts with debit/credit accounts and amounts.
    """
    elimination_entries: list[dict[str, Any]] = []

    for txn in transactions:
        txn_type = txn["type"]
        amount = _d(txn["amount"])
        from_entity = txn["from_entity"]
        to_entity = txn["to_entity"]

        if txn_type == "sales":
            elimination_entries.append({
                "type": "eliminate_ic_sales",
                "description": f"Eliminate IC sales from {from_entity} to {to_entity}",
                "debit_account": "Intercompany Revenue",
                "credit_account": "Intercompany Cost of Sales",
                "amount": str(amount),
                "from_entity": from_entity,
                "to_entity": to_entity,
            })

        elif txn_type == "receivable":
            elimination_entries.append({
                "type": "eliminate_ic_receivable",
                "description": f"Eliminate IC receivable/payable: {from_entity} - {to_entity}",
                "debit_account": "Intercompany Payable",
                "credit_account": "Intercompany Receivable",
                "amount": str(amount),
                "from_entity": from_entity,
                "to_entity": to_entity,
            })

        elif txn_type == "loan":
            elimination_entries.append({
                "type": "eliminate_ic_loan",
                "description": f"Eliminate IC loan: {from_entity} to {to_entity}",
                "debit_account": "Intercompany Loan Payable",
                "credit_account": "Intercompany Loan Receivable",
                "amount": str(amount),
                "from_entity": from_entity,
                "to_entity": to_entity,
            })

        elif txn_type == "inventory_profit":
            profit_margin = _d(txn.get("profit_margin", 0))
            unsold_pct = _d(txn.get("unsold_pct", 0))
            unrealized_profit = amount * profit_margin / Decimal("100") * unsold_pct / Decimal("100")
            elimination_entries.append({
                "type": "eliminate_ic_inventory_profit",
                "description": f"Eliminate unrealized IC inventory profit: {from_entity} -> {to_entity}",
                "debit_account": "Cost of Sales",
                "credit_account": "Inventory",
                "amount": str(unrealized_profit),
                "from_entity": from_entity,
                "to_entity": to_entity,
            })

        elif txn_type == "interest":
            elimination_entries.append({
                "type": "eliminate_ic_interest",
                "description": f"Eliminate IC interest: {from_entity} - {to_entity}",
                "debit_account": "Intercompany Interest Income",
                "credit_account": "Intercompany Interest Expense",
                "amount": str(amount),
                "from_entity": from_entity,
                "to_entity": to_entity,
            })

    return elimination_entries

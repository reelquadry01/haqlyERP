"""Government/public sector industry profile.

Author: Quadri Atharu

Covers Nigerian government entities with appropriations, IGR,
personnel costs, budget execution, and key public sector KPIs
using IPSAS accrual basis.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class GovernmentProfile:
    name: str = "Government"
    typical_revenue_models: dict[str, str] = field(default_factory=lambda: {
        "appropriations": "Statutory allocations (FAAC, federation account)",
        "igr": "Internally generated revenue (taxes, fines, fees)",
        "grants": "Grants from development partners and donor agencies",
        "capital_receipts": "Capital receipts and privatization proceeds",
    })
    cost_structures: dict[str, str] = field(default_factory=lambda: {
        "personnel": "Personnel costs (salaries, pensions, allowances)",
        "overhead": "Overhead costs (utilities, travel, stationery)",
        "capital": "Capital expenditure (projects, infrastructure)",
        "debt_service": "Debt service (interest and principal repayment)",
        "transfers": "Statutory transfers and subventions",
    })
    tax_implications: dict[str, Decimal] = field(default_factory=lambda: {
        "tax_exempt": Decimal("0"),
    })
    compliance_requirements: list[str] = field(default_factory=lambda: [
        "IPSAS accrual basis compliance",
        "Fiscal Responsibility Act compliance",
        "Public Procurement Act compliance",
        "Auditor-General audit requirements",
        "Budget Office of Federation reporting",
        "Treasury Single Account (TSA) compliance",
        "ICPC/EFCC anti-corruption compliance",
    ])
    inventory_logic: str = "Consumption method for consumables. Fixed assets tracked in asset register per IPSAS."
    depreciation_patterns: dict[str, str] = field(default_factory=lambda: {
        "infrastructure": "Straight-line over 30-50 years",
        "buildings": "Straight-line over 25-50 years",
        "vehicles": "Straight-line over 4-5 years",
        "it_equipment": "Straight-line over 3-5 years",
    })
    typical_coa_ranges: dict[str, tuple[int, int]] = field(default_factory=lambda: {
        "cash_tsa": (1000, 1099),
        "receivables": (1100, 1299),
        "ppe": (2100, 2499),
        "payables": (3100, 3299),
        "revenue": (4100, 4499),
        "personnel": (5100, 5199),
        "overhead": (5200, 5399),
        "capital_exp": (5400, 5599),
    })
    key_kpis: list[str] = field(default_factory=lambda: [
        "budget_execution_ratio",
        "revenue_collection_efficiency",
        "personnel_cost_ratio",
    ])

    def get_posting_suggestions(self, transaction: dict[str, Any]) -> list[dict[str, str]]:
        txn_type = transaction.get("type", "")
        posting_map: dict[str, list[dict[str, str]]] = {
            "appropriation_received": [
                {"debit_account": "Bank (TSA)", "credit_account": "Statutory Allocation Revenue"},
            ],
            "igr_collected": [
                {"debit_account": "Bank (TSA)", "credit_account": "IGR Revenue"},
            ],
            "salary_payment": [
                {"debit_account": "Personnel Cost", "credit_account": "Bank (TSA)"},
            ],
            "capital_project": [
                {"debit_account": "Capital Expenditure", "credit_account": "Bank (TSA) / AP"},
            ],
            "overhead": [
                {"debit_account": "Overhead Expense", "credit_account": "Bank (TSA)"},
            ],
        }
        return posting_map.get(txn_type, [{"debit_account": "Expenditure", "credit_account": "Bank (TSA) / AP"}])

    def get_industry_kpis(self, financial_data: dict[str, Any]) -> dict[str, Decimal]:
        actual_spending = _d(financial_data.get("actual_spending", 0))
        budgeted_spending = _d(financial_data.get("budgeted_spending", 1))
        igr_actual = _d(financial_data.get("igr_actual", 0))
        igr_target = _d(financial_data.get("igr_target", 1))
        personnel_cost = _d(financial_data.get("personnel_cost", 0))
        total_revenue = _d(financial_data.get("total_revenue", 1))

        budget_exec = (actual_spending / budgeted_spending * Decimal("100")).quantize(TWO_PLACES) if budgeted_spending > 0 else Decimal("0")
        collection_eff = (igr_actual / igr_target * Decimal("100")).quantize(TWO_PLACES) if igr_target > 0 else Decimal("0")
        personnel_ratio = (personnel_cost / total_revenue * Decimal("100")).quantize(TWO_PLACES) if total_revenue > 0 else Decimal("0")

        return {
            "budget_execution_ratio_pct": budget_exec,
            "revenue_collection_efficiency_pct": collection_eff,
            "personnel_cost_ratio_pct": personnel_ratio,
        }

"""NGO / Non-profit industry profile.

Author: Quadri Atharu

Covers Nigerian NGOs with donor grants, program costs, admin/fundraising
ratios, CIT exemption, and key NGO KPIs.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class NGOProfile:
    name: str = "NGO"
    typical_revenue_models: dict[str, str] = field(default_factory=lambda: {
        "donor_grants": "Restricted and unrestricted grants from donors",
        "contributions": "Individual and corporate contributions",
        "membership_fees": "Membership subscription fees",
        "investment_income": "Income from endowment investments",
    })
    cost_structures: dict[str, str] = field(default_factory=lambda: {
        "program_costs": "Direct program implementation costs",
        "admin_costs": "Administration and governance costs",
        "fundraising_costs": "Fundraising and resource mobilization costs",
        "compliance_costs": "Regulatory and reporting compliance costs",
    })
    tax_implications: dict[str, Decimal] = field(default_factory=lambda: {
        "cit_exempt_registered": Decimal("0"),
        "vat": Decimal("7.5"),
    })
    compliance_requirements: list[str] = field(default_factory=lambda: {
        "CAC (Special) registration as IT/NGO",
        "SCUML registration (Special Control Unit against ML)",
        "FIRS tax exemption certificate",
        "Donor reporting requirements (USAID, DFID, EU)",
        "Annual returns to CAC",
        "NFVCB compliance (if applicable)",
    })
    inventory_logic: str = "N/A - NGOs typically do not carry saleable inventory. Relief items tracked as program materials."
    depreciation_patterns: dict[str, str] = field(default_factory=lambda: {
        "office_equipment": "Straight-line over 3-5 years",
        "vehicles": "Straight-line over 4-5 years",
        "it_systems": "Straight-line over 3-5 years",
    })
    typical_coa_ranges: dict[str, tuple[int, int]] = field(default_factory=lambda: {
        "cash": (1000, 1099),
        "receivables": (1100, 1199),
        "ppe": (2100, 2199),
        "payables": (3100, 3199),
        "grant_revenue": (4100, 4199),
        "program_costs": (5100, 5399),
        "admin_costs": (5400, 5499),
    })
    key_kpis: list[str] = field(default_factory=lambda: [
        "program_spend_ratio",
        "admin_cost_ratio",
        "donor_retention",
        "grant_utilization",
    ])

    def get_posting_suggestions(self, transaction: dict[str, Any]) -> list[dict[str, str]]:
        txn_type = transaction.get("type", "")
        posting_map: dict[str, list[dict[str, str]]] = {
            "grant_received": [
                {"debit_account": "Bank", "credit_account": "Grant Revenue (Restricted/Unrestricted)"},
            ],
            "contribution": [
                {"debit_account": "Bank", "credit_account": "Contributions Revenue"},
            ],
            "program_cost": [
                {"debit_account": "Program Costs", "credit_account": "Bank / AP"},
            ],
            "admin_cost": [
                {"debit_account": "Administration Costs", "credit_account": "Bank / AP"},
            ],
            "fundraising_cost": [
                {"debit_account": "Fundraising Costs", "credit_account": "Bank / AP"},
            ],
            "grant_refund": [
                {"debit_account": "Grant Revenue", "credit_account": "Bank"},
            ],
        }
        return posting_map.get(txn_type, [{"debit_account": "Expense", "credit_account": "Bank / AP"}])

    def get_industry_kpis(self, financial_data: dict[str, Any]) -> dict[str, Decimal]:
        program_spend = _d(financial_data.get("program_spend", 0))
        total_spend = _d(financial_data.get("total_spend", 1))
        admin_spend = _d(financial_data.get("admin_spend", 0))
        donors_retained = _d(financial_data.get("donors_retained", 0))
        total_donors = _d(financial_data.get("total_donors", 1))
        grants_utilized = _d(financial_data.get("grants_utilized", 0))
        grants_received = _d(financial_data.get("grants_received", 1))

        program_ratio = (program_spend / total_spend * Decimal("100")).quantize(TWO_PLACES) if total_spend > 0 else Decimal("0")
        admin_ratio = (admin_spend / total_spend * Decimal("100")).quantize(TWO_PLACES) if total_spend > 0 else Decimal("0")
        donor_ret = (donors_retained / total_donors * Decimal("100")).quantize(TWO_PLACES) if total_donors > 0 else Decimal("0")
        grant_util = (grants_utilized / grants_received * Decimal("100")).quantize(TWO_PLACES) if grants_received > 0 else Decimal("0")

        return {
            "program_spend_ratio_pct": program_ratio,
            "admin_cost_ratio_pct": admin_ratio,
            "donor_retention_pct": donor_ret,
            "grant_utilization_pct": grant_util,
        }

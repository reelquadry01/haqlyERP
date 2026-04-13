"""Construction industry profile.

Author: Quadri Atharu

Covers Nigerian construction with POC revenue recognition, contract
costs, WHT on contracts, and key construction KPIs per IFRS 15.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class ConstructionProfile:
    name: str = "Construction"
    typical_revenue_models: dict[str, str] = field(default_factory=lambda: {
        "contract_revenue": "Revenue from construction contracts (IFRS 15 POC)",
        "variation_orders": "Revenue from approved contract variations",
        "claims_income": "Revenue from contract claims and disputes",
        "equipment_hire": "Revenue from equipment rental to third parties",
    })
    cost_structures: dict[str, str] = field(default_factory=lambda: {
        "materials": "Construction materials (cement, steel, sand, etc.)",
        "subcontractors": "Subcontractor costs",
        "equipment": "Equipment depreciation, fuel, maintenance",
        "labor": "Site labor and supervision",
        "overheads": "Project overheads (insurance, permits, site office)",
    })
    tax_implications: dict[str, Decimal] = field(default_factory=lambda: {
        "vat": Decimal("7.5"),
        "cit": Decimal("30"),
        "wht_contract": Decimal("5"),
        "education_tax": Decimal("2"),
    })
    compliance_requirements: list[str] = field(default_factory=lambda: [
        "CAC company registration",
        "BPP registration (Bureau of Public Procurement)",
        "FG/State contractor registration",
        "NESREA environmental compliance",
        "Building code compliance",
        "FIRS tax returns",
    ])
    inventory_logic: str = "Cost-to-cost method for POC. Materials on site tracked as current asset until consumed."
    depreciation_patterns: dict[str, str] = field(default_factory=lambda: {
        "heavy_equipment": "Straight-line over 5-10 years",
        "vehicles": "Straight-line over 4-5 years",
        "temporary_structures": "Over contract period",
    })
    typical_coa_ranges: dict[str, tuple[int, int]] = field(default_factory=lambda: {
        "materials_on_site": (1100, 1199),
        "contract_wip": (1200, 1399),
        "contract_assets": (1400, 1499),
        "retention": (1500, 1599),
        "contract_revenue": (6100, 6199),
        "contract_costs": (7100, 7199),
    })
    key_kpis: list[str] = field(default_factory=lambda: [
        "project_margin",
        "backlog",
        "cost_variance",
        "schedule_performance_index",
    ])

    def get_posting_suggestions(self, transaction: dict[str, Any]) -> list[dict[str, str]]:
        txn_type = transaction.get("type", "")
        posting_map: dict[str, list[dict[str, str]]] = {
            "contract_revenue": [
                {"debit_account": "Contract Asset / Receivables", "credit_account": "Contract Revenue"},
            ],
            "material_purchase": [
                {"debit_account": "Materials on Site", "credit_account": "Bank / AP"},
            ],
            "subcontractor_cost": [
                {"debit_account": "Contract Cost - Subcontractors", "credit_account": "Bank / Subcontractor Payable"},
            ],
            "equipment_cost": [
                {"debit_account": "Contract Cost - Equipment", "credit_account": "Bank / Equipment Depreciation"},
            ],
            "variation_order": [
                {"debit_account": "Contract Asset", "credit_account": "Contract Revenue - Variations"},
            ],
            "progress_certified": [
                {"debit_account": "Receivables - Retention", "credit_account": "Contract Asset"},
            ],
        }
        return posting_map.get(txn_type, [{"debit_account": "Operating Expense", "credit_account": "Bank / AP"}])

    def get_industry_kpis(self, financial_data: dict[str, Any]) -> dict[str, Decimal]:
        contract_profit = _d(financial_data.get("contract_profit", 0))
        contract_revenue = _d(financial_data.get("contract_revenue", 1))
        backlog_value = _d(financial_data.get("backlog_value", 0))
        budgeted_cost = _d(financial_data.get("budgeted_cost", 1))
        actual_cost = _d(financial_data.get("actual_cost", 0))
        earned_value = _d(financial_data.get("earned_value", 0))
        planned_value = _d(financial_data.get("planned_value", 1))

        margin = (contract_profit / contract_revenue * Decimal("100")).quantize(TWO_PLACES) if contract_revenue > 0 else Decimal("0")
        variance = ((budgeted_cost - actual_cost) / budgeted_cost * Decimal("100")).quantize(TWO_PLACES) if budgeted_cost > 0 else Decimal("0")
        spi = (earned_value / planned_value).quantize(TWO_PLACES) if planned_value > 0 else Decimal("0")

        return {
            "project_margin_pct": margin,
            "backlog": backlog_value,
            "cost_variance_pct": variance,
            "schedule_performance_index": spi,
        }

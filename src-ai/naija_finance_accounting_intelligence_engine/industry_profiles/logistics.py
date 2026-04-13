"""Logistics industry profile.

Author: Quadri Atharu

Covers Nigerian logistics with freight, warehousing, last-mile
revenue, fleet costs, and key logistics KPIs.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class LogisticsProfile:
    name: str = "Logistics"
    typical_revenue_models: dict[str, str] = field(default_factory=lambda: {
        "freight_revenue": "Revenue from freight/haulage services",
        "warehousing": "Revenue from warehousing and storage",
        "last_mile": "Revenue from last-mile delivery",
        "clearing_forwarding": "Revenue from customs clearing and forwarding",
    })
    cost_structures: dict[str, str] = field(default_factory=lambda: {
        "fuel": "Fuel and diesel costs",
        "maintenance": "Fleet maintenance and repairs",
        "fleet_depreciation": "Vehicle and equipment depreciation",
        "tolls_tariffs": "Road tolls and port tariffs",
        "labor": "Driver wages and logistics staff",
    })
    tax_implications: dict[str, Decimal] = field(default_factory=lambda: {
        "vat": Decimal("7.5"),
        "cit": Decimal("30"),
        "wht_contract": Decimal("5"),
        "education_tax": Decimal("2"),
    })
    compliance_requirements: list[str] = field(default_factory=lambda: [
        "CAC company registration",
        "NURTW/regulatory permits",
        "Customs licence (clearing agents)",
        "Vehicle inspection and roadworthiness",
        "FIRS tax returns",
    ])
    inventory_logic: str = "N/A - Logistics companies do not carry saleable inventory"
    depreciation_patterns: dict[str, str] = field(default_factory=lambda: {
        "trucks": "Units of production based on kilometres or hours",
        "warehouse": "Straight-line over 20-30 years",
        "equipment": "Straight-line over 5-10 years",
    })
    typical_coa_ranges: dict[str, tuple[int, int]] = field(default_factory=lambda: {
        "fleet_assets": (2100, 2299),
        "warehouse_assets": (2300, 2399),
        "revenue": (6100, 6299),
        "fleet_costs": (7100, 7299),
    })
    key_kpis: list[str] = field(default_factory=lambda: [
        "fleet_utilization",
        "on_time_delivery",
        "cost_per_km",
        "fuel_efficiency",
    ])

    def get_posting_suggestions(self, transaction: dict[str, Any]) -> list[dict[str, str]]:
        txn_type = transaction.get("type", "")
        posting_map: dict[str, list[dict[str, str]]] = {
            "freight_revenue": [
                {"debit_account": "Bank / Receivables", "credit_account": "Freight Revenue"},
            ],
            "fuel_purchase": [
                {"debit_account": "Fuel Expense", "credit_account": "Bank / AP"},
            ],
            "maintenance": [
                {"debit_account": "Fleet Maintenance Expense", "credit_account": "Bank / AP"},
            ],
            "toll_payment": [
                {"debit_account": "Tolls & Tariffs Expense", "credit_account": "Bank"},
            ],
            "warehouse_revenue": [
                {"debit_account": "Bank / Receivables", "credit_account": "Warehousing Revenue"},
            ],
            "driver_wages": [
                {"debit_account": "Labor Expense - Drivers", "credit_account": "Bank / Payroll Payable"},
            ],
        }
        return posting_map.get(txn_type, [{"debit_account": "Operating Expense", "credit_account": "Bank / AP"}])

    def get_industry_kpis(self, financial_data: dict[str, Any]) -> dict[str, Decimal]:
        active_fleet_hours = _d(financial_data.get("active_fleet_hours", 0))
        total_fleet_hours = _d(financial_data.get("total_fleet_hours", 1))
        deliveries_on_time = _d(financial_data.get("deliveries_on_time", 0))
        total_deliveries = _d(financial_data.get("total_deliveries", 1))
        total_fleet_cost = _d(financial_data.get("total_fleet_cost", 0))
        total_km = _d(financial_data.get("total_km", 1))
        fuel_cost = _d(financial_data.get("fuel_cost", 0))
        km_travelled = _d(financial_data.get("km_travelled", 1))

        utilization = (active_fleet_hours / total_fleet_hours * Decimal("100")).quantize(TWO_PLACES) if total_fleet_hours > 0 else Decimal("0")
        on_time = (deliveries_on_time / total_deliveries * Decimal("100")).quantize(TWO_PLACES) if total_deliveries > 0 else Decimal("0")
        cost_km = (total_fleet_cost / total_km).quantize(TWO_PLACES) if total_km > 0 else Decimal("0")
        fuel_eff = (fuel_cost / km_travelled * Decimal("100")).quantize(TWO_PLACES) if km_travelled > 0 else Decimal("0")

        return {
            "fleet_utilization_pct": utilization,
            "on_time_delivery_pct": on_time,
            "cost_per_km": cost_km,
            "fuel_efficiency_pct_of_cost": fuel_eff,
        }

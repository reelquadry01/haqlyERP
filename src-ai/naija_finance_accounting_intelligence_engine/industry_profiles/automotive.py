"""Automotive industry profile.

Author: Quadri Atharu

Covers Nigerian automotive with vehicle sales, parts, service revenue,
declining balance depreciation, and key automotive KPIs.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class AutomotiveProfile:
    name: str = "Automotive"
    typical_revenue_models: dict[str, str] = field(default_factory=lambda: {
        "vehicle_sales": "Revenue from new and used vehicle sales",
        "parts_sales": "Revenue from spare parts and accessories",
        "service_revenue": "Revenue from vehicle servicing and repairs",
        "body_shop": "Revenue from body and paint work",
        "insurance_commission": "Commission from motor insurance referrals",
    })
    cost_structures: dict[str, str] = field(default_factory=lambda: {
        "cogs_vehicles": "Cost of vehicles purchased for resale",
        "cogs_parts": "Cost of spare parts",
        "warranty_costs": "Warranty and recall costs",
        "floor_plan_interest": "Interest on floor plan financing",
        "technician_labor": "Technician wages and training",
    })
    tax_implications: dict[str, Decimal] = field(default_factory=lambda: {
        "vat": Decimal("7.5"),
        "cit": Decimal("30"),
        "wht_imports": Decimal("5"),
        "import_duty": Decimal("20"),
        "education_tax": Decimal("2"),
    })
    compliance_requirements: list[str] = field(default_factory=lambda: [
        "NAFDAC/SON vehicle standards",
        "NCS import clearance",
        "Dealer franchise agreements",
        "FIRS tax returns",
        "State vehicle registration compliance",
    ])
    inventory_logic: str = "FIFO for vehicles and parts. Floor plan financing tracked as secured borrowing."
    depreciation_patterns: dict[str, str] = field(default_factory=lambda: {
        "shop_equipment": "Declining balance method",
        "vehicles_demo": "Declining balance over 2-3 years",
        "buildings": "Straight-line over 25-40 years",
    })
    typical_coa_ranges: dict[str, tuple[int, int]] = field(default_factory=lambda: {
        "vehicle_inventory": (1100, 1199),
        "parts_inventory": (1200, 1299),
        "shop_equipment": (2100, 2199),
        "floor_plan_payable": (3100, 3199),
        "vehicle_revenue": (6100, 6199),
        "parts_revenue": (6200, 6299),
        "service_revenue": (6300, 6399),
    })
    key_kpis: list[str] = field(default_factory=lambda: [
        "vehicle_turnover",
        "gross_margin_per_unit",
        "service_absorption",
        "parts_fill_rate",
    ])

    def get_posting_suggestions(self, transaction: dict[str, Any]) -> list[dict[str, str]]:
        txn_type = transaction.get("type", "")
        posting_map: dict[str, list[dict[str, str]]] = {
            "vehicle_purchase": [
                {"debit_account": "Vehicle Inventory", "credit_account": "Bank / AP / Floor Plan Payable"},
            ],
            "vehicle_sale": [
                {"debit_account": "Bank / Receivables", "credit_account": "Vehicle Sales Revenue"},
                {"debit_account": "Cost of Vehicles Sold", "credit_account": "Vehicle Inventory"},
            ],
            "parts_sale": [
                {"debit_account": "Bank / Receivables", "credit_account": "Parts Sales Revenue"},
                {"debit_account": "Cost of Parts Sold", "credit_account": "Parts Inventory"},
            ],
            "service_revenue": [
                {"debit_account": "Bank / Receivables", "credit_account": "Service Revenue"},
            ],
            "floor_plan_interest": [
                {"debit_account": "Floor Plan Interest Expense", "credit_account": "Bank / Floor Plan Payable"},
            ],
            "warranty_cost": [
                {"debit_account": "Warranty Expense", "credit_account": "Bank / AP / Parts Inventory"},
            ],
        }
        return posting_map.get(txn_type, [{"debit_account": "Operating Expense", "credit_account": "Bank / AP"}])

    def get_industry_kpis(self, financial_data: dict[str, Any]) -> dict[str, Decimal]:
        vehicles_sold = _d(financial_data.get("vehicles_sold", 0))
        avg_vehicle_inventory = _d(financial_data.get("avg_vehicle_inventory", 1))
        gross_margin_total = _d(financial_data.get("gross_margin_total", 0))
        units_sold = _d(financial_data.get("units_sold", 1))
        service_gross_profit = _d(financial_data.get("service_gross_profit", 0))
        total_fixed_overhead = _d(financial_data.get("total_fixed_overhead", 1))
        parts_filled = _d(financial_data.get("parts_filled", 0))
        parts_requested = _d(financial_data.get("parts_requested", 1))

        turnover = (vehicles_sold / avg_vehicle_inventory).quantize(TWO_PLACES) if avg_vehicle_inventory > 0 else Decimal("0")
        margin_unit = (gross_margin_total / units_sold).quantize(TWO_PLACES) if units_sold > 0 else Decimal("0")
        absorption = (service_gross_profit / total_fixed_overhead * Decimal("100")).quantize(TWO_PLACES) if total_fixed_overhead > 0 else Decimal("0")
        fill_rate = (parts_filled / parts_requested * Decimal("100")).quantize(TWO_PLACES) if parts_requested > 0 else Decimal("0")

        return {
            "vehicle_turnover": turnover,
            "gross_margin_per_unit": margin_unit,
            "service_absorption_pct": absorption,
            "parts_fill_rate_pct": fill_rate,
        }

"""Manufacturing industry profile.

Author: Quadri Atharu

Covers Nigerian manufacturing with product sales revenue, raw material
and labor cost structures, CIT and VAT tax implications, FIFO/weighted
average inventory, straight-line depreciation, and key KPIs.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class ManufacturingProfile:
    name: str = "Manufacturing"
    typical_revenue_models: dict[str, str] = field(default_factory=lambda: {
        "product_sales": "Revenue from manufactured goods",
        "contract_manufacturing": "Toll/contract manufacturing fees",
        "by_product_sales": "Revenue from manufacturing by-products",
        "scrap_sales": "Revenue from scrap material sales",
    })
    cost_structures: dict[str, str] = field(default_factory=lambda: {
        "raw_materials": "Direct material costs",
        "direct_labor": "Direct labor costs",
        "manufacturing_overhead": "Factory overhead (utilities, depreciation, maintenance)",
        "administrative_overhead": "Admin and management costs",
        "selling_distribution": "Selling and distribution costs",
    })
    tax_implications: dict[str, Decimal] = field(default_factory=lambda: {
        "cit": Decimal("30"),
        "vat": Decimal("7.5"),
        "wht_services": Decimal("5"),
        "wht_contract": Decimal("5"),
        "education_tax": Decimal("2"),
    })
    compliance_requirements: list[str] = field(default_factory=lambda: [
        "SON/MANCAP product certification",
        "NAFDAC registration (food/drugs/cosmetics)",
        "NESREA environmental compliance",
        "Factory inspection and OSH compliance",
        "FIRS CIT and VAT returns",
        "Pioneer status incentive application (if eligible)",
    ])
    inventory_logic: str = "FIFO or Weighted Average for raw materials and finished goods. Work-in-progress valued at cost incurred to date."
    depreciation_patterns: dict[str, str] = field(default_factory=lambda: {
        "plant_machinery": "Straight-line over 5-10 years",
        "buildings": "Straight-line over 20-50 years",
        "vehicles": "Straight-line over 4-5 years",
        "equipment": "Straight-line over 5-7 years",
    })
    typical_coa_ranges: dict[str, tuple[int, int]] = field(default_factory=lambda: {
        "raw_materials": (1100, 1199),
        "work_in_progress": (1200, 1299),
        "finished_goods": (1300, 1399),
        "plant_machinery": (2100, 2199),
        "accumulated_dep": (2200, 2299),
        "trade_payables": (3100, 3199),
        "revenue": (6100, 6199),
        "cogs": (7100, 7199),
        "mfg_overhead": (7200, 7299),
    })
    key_kpis: list[str] = field(default_factory=lambda: [
        "capacity_utilization",
        "cost_per_unit",
        "scrap_rate",
        "inventory_turnover",
        "oee",
    ])

    def get_posting_suggestions(self, transaction: dict[str, Any]) -> list[dict[str, str]]:
        txn_type = transaction.get("type", "")
        posting_map: dict[str, list[dict[str, str]]] = {
            "raw_material_purchase": [
                {"debit_account": "Raw Material Inventory", "credit_account": "Bank / AP"},
            ],
            "direct_labor": [
                {"debit_account": "Work in Progress", "credit_account": "Bank / Payroll Payable"},
            ],
            "mfg_overhead": [
                {"debit_account": "Manufacturing Overhead", "credit_account": "Bank / AP / Accrued Expenses"},
            ],
            "finished_goods": [
                {"debit_account": "Finished Goods Inventory", "credit_account": "Work in Progress"},
            ],
            "product_sale": [
                {"debit_account": "Bank / Receivables", "credit_account": "Sales Revenue"},
                {"debit_account": "Cost of Goods Sold", "credit_account": "Finished Goods Inventory"},
            ],
            "scrap_sale": [
                {"debit_account": "Bank", "credit_account": "Other Income - Scrap Sales"},
            ],
            "vat_output": [
                {"debit_account": "Bank / Receivables", "credit_account": "VAT Output Payable"},
            ],
            "vat_input": [
                {"debit_account": "VAT Input Receivable", "credit_account": "Bank / AP"},
            ],
        }
        return posting_map.get(txn_type, [{"debit_account": "Operating Expense", "credit_account": "Bank / AP"}])

    def get_industry_kpis(self, financial_data: dict[str, Any]) -> dict[str, Decimal]:
        actual_output = _d(financial_data.get("actual_output", 0))
        max_capacity = _d(financial_data.get("max_capacity", 1))
        total_cost = _d(financial_data.get("total_cost", 0))
        units_produced = _d(financial_data.get("units_produced", 1))
        scrap_units = _d(financial_data.get("scrap_units", 0))
        total_units = _d(financial_data.get("total_units", 1))
        cogs = _d(financial_data.get("cogs", 0))
        avg_inventory = _d(financial_data.get("avg_inventory", 1))
        availability = _d(financial_data.get("availability_pct", 100))
        performance = _d(financial_data.get("performance_pct", 100))
        quality = _d(financial_data.get("quality_pct", 100))

        capacity_util = (actual_output / max_capacity * Decimal("100")).quantize(TWO_PLACES) if max_capacity > 0 else Decimal("0")
        cost_per_unit = (total_cost / units_produced).quantize(TWO_PLACES) if units_produced > 0 else Decimal("0")
        scrap_rate = (scrap_units / total_units * Decimal("100")).quantize(TWO_PLACES) if total_units > 0 else Decimal("0")
        inv_turnover = (cogs / avg_inventory).quantize(TWO_PLACES) if avg_inventory > 0 else Decimal("0")
        oee = (availability * performance * quality / Decimal("10000")).quantize(TWO_PLACES)

        return {
            "capacity_utilization_pct": capacity_util,
            "cost_per_unit": cost_per_unit,
            "scrap_rate_pct": scrap_rate,
            "inventory_turnover": inv_turnover,
            "oee_pct": oee,
        }

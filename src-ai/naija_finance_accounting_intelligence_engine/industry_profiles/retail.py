"""Retail industry profile.

Author: Quadri Atharu

Covers Nigerian retail with store/online sales, COGS, FIFO/retail
inventory method, and key retail KPIs.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class RetailProfile:
    name: str = "Retail"
    typical_revenue_models: dict[str, str] = field(default_factory=lambda: {
        "store_sales": "In-store merchandise sales",
        "online_sales": "E-commerce and digital channel sales",
        "commission_income": "Commission from third-party products",
        "loyalty_program": "Revenue from loyalty/subscription programs",
    })
    cost_structures: dict[str, str] = field(default_factory=lambda: {
        "cogs": "Cost of goods sold (purchase price + freight in)",
        "rent_occupancy": "Store rent, service charges, occupancy costs",
        "staffing": "Sales staff wages and benefits",
        "marketing": "Advertising, promotions, loyalty program costs",
        "logistics": "Warehousing, distribution, last-mile delivery",
    })
    tax_implications: dict[str, Decimal] = field(default_factory=lambda: {
        "vat": Decimal("7.5"),
        "cit": Decimal("30"),
        "wht_rent": Decimal("10"),
        "education_tax": Decimal("2"),
    })
    compliance_requirements: list[str] = field(default_factory=lambda: [
        "NAFDAC compliance (food/cosmetics)",
        "SON product standards",
        "FIRS VAT and CIT returns",
        "Consumer protection compliance (FCCPC)",
        "State/Local government levies",
    ])
    inventory_logic: str = "FIFO or Retail Inventory Method (RIM). Mark-downs recognized immediately."
    depreciation_patterns: dict[str, str] = field(default_factory=lambda: {
        "shop_fixtures": "Straight-line over 5-7 years",
        "it_pos": "Straight-line over 3-5 years",
        "vehicles": "Straight-line over 4-5 years",
    })
    typical_coa_ranges: dict[str, tuple[int, int]] = field(default_factory=lambda: {
        "inventory": (1100, 1299),
        "receivables": (1300, 1399),
        "payables": (3100, 3199),
        "sales": (6100, 6299),
        "cogs": (7100, 7199),
        "selling_expenses": (7200, 7399),
    })
    key_kpis: list[str] = field(default_factory=lambda: [
        "sales_per_sqm",
        "gross_margin",
        "inventory_turnover",
        "footfall_conversion",
    ])

    def get_posting_suggestions(self, transaction: dict[str, Any]) -> list[dict[str, str]]:
        txn_type = transaction.get("type", "")
        posting_map: dict[str, list[dict[str, str]]] = {
            "merchandise_purchase": [
                {"debit_account": "Inventory", "credit_account": "Bank / AP"},
            ],
            "store_sale": [
                {"debit_account": "Bank / Receivables", "credit_account": "Sales Revenue"},
                {"debit_account": "Cost of Goods Sold", "credit_account": "Inventory"},
            ],
            "online_sale": [
                {"debit_account": "Bank", "credit_account": "Online Sales Revenue"},
                {"debit_account": "Cost of Goods Sold", "credit_account": "Inventory"},
            ],
            "rent_payment": [
                {"debit_account": "Rent & Occupancy Expense", "credit_account": "Bank"},
            ],
            "staff_cost": [
                {"debit_account": "Staff Costs", "credit_account": "Bank / Payroll Payable"},
            ],
            "mark_down": [
                {"debit_account": "Mark-down Expense", "credit_account": "Inventory"},
            ],
        }
        return posting_map.get(txn_type, [{"debit_account": "Operating Expense", "credit_account": "Bank / AP"}])

    def get_industry_kpis(self, financial_data: dict[str, Any]) -> dict[str, Decimal]:
        total_sales = _d(financial_data.get("total_sales", 0))
        store_area_sqm = _d(financial_data.get("store_area_sqm", 1))
        gross_profit = _d(financial_data.get("gross_profit", 0))
        cogs = _d(financial_data.get("cogs", 0))
        avg_inventory = _d(financial_data.get("avg_inventory", 1))
        footfall = _d(financial_data.get("footfall", 1))
        transactions = _d(financial_data.get("transactions", 0))

        sales_per_sqm = (total_sales / store_area_sqm).quantize(TWO_PLACES) if store_area_sqm > 0 else Decimal("0")
        gross_margin = (gross_profit / total_sales * Decimal("100")).quantize(TWO_PLACES) if total_sales > 0 else Decimal("0")
        inv_turnover = (cogs / avg_inventory).quantize(TWO_PLACES) if avg_inventory > 0 else Decimal("0")
        conversion = (transactions / footfall * Decimal("100")).quantize(TWO_PLACES) if footfall > 0 else Decimal("0")

        return {
            "sales_per_sqm": sales_per_sqm,
            "gross_margin_pct": gross_margin,
            "inventory_turnover": inv_turnover,
            "footfall_conversion_pct": conversion,
        }

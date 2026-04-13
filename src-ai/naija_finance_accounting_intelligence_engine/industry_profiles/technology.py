"""Technology industry profile.

Author: Quadri Atharu

Covers Nigerian tech with SaaS, licensing, consulting revenue,
R&D costs, pioneer status incentives, and key SaaS KPIs.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class TechnologyProfile:
    name: str = "Technology"
    typical_revenue_models: dict[str, str] = field(default_factory=lambda: {
        "saas_subscriptions": "Monthly/annual SaaS subscription fees",
        "licensing": "Software licensing revenue (perpetual/term)",
        "consulting": "IT consulting and implementation services",
        "api_platform": "API access and platform usage fees",
    })
    cost_structures: dict[str, str] = field(default_factory=lambda: {
        "rd": "Research and development costs",
        "cloud_infra": "Cloud infrastructure (AWS, Azure, GCP)",
        "talent": "Engineering and product talent costs",
        "sales_marketing": "Customer acquisition and marketing",
    })
    tax_implications: dict[str, Decimal] = field(default_factory=lambda: {
        "vat": Decimal("7.5"),
        "cit": Decimal("30"),
        "pioneer_status": Decimal("0"),
        "education_tax": Decimal("2"),
    })
    compliance_requirements: list[str] = field(default_factory=lambda: [
        "NITDA registration",
        "NCC licence (if telecom-related)",
        "NDPR/NDPA data protection compliance",
        "FIRS tax returns",
        "Pioneer status application (if eligible)",
        "NIPC registration for pioneer status",
    ])
    inventory_logic: str = "N/A - Software companies do not carry physical inventory. Capitalized software development costs under IAS 38."
    depreciation_patterns: dict[str, str] = field(default_factory=lambda: {
        "software_development": "Amortized over 3-5 years (IAS 38)",
        "servers": "Straight-line over 3-5 years",
        "office_equipment": "Straight-line over 3-5 years",
    })
    typical_coa_ranges: dict[str, tuple[int, int]] = field(default_factory=lambda: {
        "intangibles": (2100, 2299),
        "revenue": (6100, 6299),
        "rd_costs": (7100, 7199),
        "cloud_costs": (7200, 7299),
    })
    key_kpis: list[str] = field(default_factory=lambda: [
        "arr_growth",
        "churn_rate",
        "cac",
        "ltv",
        "gross_margin",
    ])

    def get_posting_suggestions(self, transaction: dict[str, Any]) -> list[dict[str, str]]:
        txn_type = transaction.get("type", "")
        posting_map: dict[str, list[dict[str, str]]] = {
            "saas_subscription": [
                {"debit_account": "Bank / Receivables", "credit_account": "SaaS Subscription Revenue"},
            ],
            "license_revenue": [
                {"debit_account": "Bank / Receivables", "credit_account": "License Revenue"},
            ],
            "consulting_revenue": [
                {"debit_account": "Bank / Receivables", "credit_account": "Consulting Revenue"},
            ],
            "rd_expense": [
                {"debit_account": "R&D Expense", "credit_account": "Bank / AP"},
            ],
            "cloud_infra": [
                {"debit_account": "Cloud Infrastructure Expense", "credit_account": "Bank / AP"},
            ],
            "talent_cost": [
                {"debit_account": "Engineering Talent Costs", "credit_account": "Bank / Payroll Payable"},
            ],
            "capitalized_dev": [
                {"debit_account": "Capitalized Software Development (IAS 38)", "credit_account": "Bank / AP / Payroll"},
            ],
        }
        return posting_map.get(txn_type, [{"debit_account": "Operating Expense", "credit_account": "Bank / AP"}])

    def get_industry_kpis(self, financial_data: dict[str, Any]) -> dict[str, Decimal]:
        arr_current = _d(financial_data.get("arr_current", 0))
        arr_prior = _d(financial_data.get("arr_prior", 1))
        customers_lost = _d(financial_data.get("customers_lost", 0))
        customers_start = _d(financial_data.get("customers_start", 1))
        sales_marketing_cost = _d(financial_data.get("sales_marketing_cost", 0))
        new_customers = _d(financial_data.get("new_customers", 1))
        arpu = _d(financial_data.get("arpu", 0))
        churn_rate = _d(financial_data.get("churn_monthly_pct", 0))
        gross_profit = _d(financial_data.get("gross_profit", 0))
        revenue = _d(financial_data.get("revenue", 1))

        arr_growth = ((arr_current - arr_prior) / arr_prior * Decimal("100")).quantize(TWO_PLACES) if arr_prior > 0 else Decimal("0")
        churn = (customers_lost / customers_start * Decimal("100")).quantize(TWO_PLACES) if customers_start > 0 else Decimal("0")
        cac = (sales_marketing_cost / new_customers).quantize(TWO_PLACES) if new_customers > 0 else Decimal("0")
        ltv = (arpu / (churn_rate / Decimal("100"))) if churn_rate > 0 else arpu * Decimal("36")
        ltv = ltv.quantize(TWO_PLACES)
        gross_margin = (gross_profit / revenue * Decimal("100")).quantize(TWO_PLACES) if revenue > 0 else Decimal("0")

        return {
            "arr_growth_pct": arr_growth,
            "churn_rate_pct": churn,
            "cac": cac,
            "ltv": ltv,
            "gross_margin_pct": gross_margin,
        }

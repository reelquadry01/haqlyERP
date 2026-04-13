"""Oil and Gas industry profile - upstream and downstream operations.

Author: Quadri Atharu

Covers upstream (joint ventures, PSC, marginal fields) and downstream
(refining, distribution) with Nigerian-specific tax treatments including
Petroleum Profit Tax (PPT 50%/85% for PSC), NDDC levy (3%), education
tax (2%), and unit-of-production depreciation.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class OilGasProfile:
    name: str = "Oil & Gas"
    typical_revenue_models: dict[str, str] = field(default_factory=lambda: {
        "crude_oil_sales": "Revenue from crude oil lifting and sales",
        "natural_gas_sales": "Revenue from gas production and sales",
        "refining_margins": "Margin from refining crude to petroleum products",
        "product_distribution": "Revenue from distributing refined products",
        "gas_processing": "Revenue from gas processing and LPG extraction",
        "marginal_field_production": "Revenue from marginal field operations",
    })
    cost_structures: dict[str, str] = field(default_factory=lambda: {
        "opex": "Operating expenditure - well maintenance, lifting, processing",
        "capex": "Capital expenditure - drilling, facilities, pipelines",
        "royalty": "Royalty payments to NNPC/FGN (typically 20% for onshore)",
        "psc_cost_oil": "Production Sharing Contract cost oil recovery",
        "abandonment_decommissioning": "Asset retirement obligations",
        "community_development": "Host community development obligations (3% NDDC)",
    })
    tax_implications: dict[str, Decimal] = field(default_factory=lambda: {
        "ppt_jv": Decimal("50"),
        "ppt_psc": Decimal("85"),
        "nddc_levy": Decimal("3"),
        "education_tax": Decimal("2"),
        "vat": Decimal("7.5"),
        "wht_services": Decimal("5"),
        "wht_contract": Decimal("5"),
    })
    compliance_requirements: list[str] = field(default_factory=lambda: [
        "DPR/NUIMS operational permit",
        "NNPC/NPDC joint venture agreement",
        "Environmental Impact Assessment (EIA)",
        "NUPRC regulatory compliance",
        "FIR PPT filing and assessment",
        "Flare Gas regulations compliance",
        "Host community development (3% NDDC)",
        "NERC gas pricing compliance",
        "NOSDRA oil spill response",
    ])
    inventory_logic: str = "Volume-based: crude oil inventory measured in barrels, refined products by volume. FIFO for product valuation."
    depreciation_patterns: dict[str, str] = field(default_factory=lambda: {
        "oil_gas_assets": "Unit of Production (UOP) based on proven reserves",
        "refining_equipment": "Straight-line over useful life (20-25 years)",
        "pipelines": "Straight-line or UOP based on throughput capacity",
        "intangibles": "Straight-line over contractual or useful life",
    })
    typical_coa_ranges: dict[str, tuple[int, int]] = field(default_factory=lambda: {
        "current_assets": (1000, 1999),
        "fixed_assets": (2000, 2999),
        "current_liabilities": (3000, 3999),
        "long_term_liabilities": (4000, 4999),
        "equity": (5000, 5999),
        "revenue": (6000, 6999),
        "cost_of_sales": (7000, 7499),
        "operating_expenses": (7500, 8499),
        "other_income_expense": (8500, 8999),
        "tax": (9000, 9999),
    })
    key_kpis: list[str] = field(default_factory=lambda: [
        "reserve_replacement_ratio",
        "finding_cost",
        "lifting_cost",
        "netback",
        "production_efficiency",
        "reserve_life_index",
        "finding_development_cost",
    ])

    def get_posting_suggestions(self, transaction: dict[str, Any]) -> list[dict[str, str]]:
        """Suggest debit/credit accounts for oil and gas transactions.

        Args:
            transaction: Dict with 'type' (crude_sale, gas_sale, royalty,
                        capex_drilling, opex_maintenance, psc_cost_oil,
                        product_sale, nddc_levy) and optional 'amount'.

        Returns:
            List of dicts with 'debit_account' and 'credit_account'.
        """
        txn_type = transaction.get("type", "")
        suggestions: list[dict[str, str]] = []

        posting_map: dict[str, list[dict[str, str]]] = {
            "crude_sale": [
                {"debit_account": "Bank / Receivables", "credit_account": "Crude Oil Revenue"},
            ],
            "gas_sale": [
                {"debit_account": "Bank / Receivables", "credit_account": "Gas Revenue"},
            ],
            "royalty": [
                {"debit_account": "Royalty Expense", "credit_account": "Bank / Royalty Payable"},
            ],
            "capex_drilling": [
                {"debit_account": "Oil & Gas Properties (CAPEX)", "credit_account": "Bank / AP"},
            ],
            "opex_maintenance": [
                {"debit_account": "Operating Expenses - Well Maintenance", "credit_account": "Bank / AP"},
            ],
            "psc_cost_oil": [
                {"debit_account": "PSC Cost Oil Recovery", "credit_account": "Oil Revenue - Cost Oil"},
            ],
            "product_sale": [
                {"debit_account": "Bank / Receivables", "credit_account": "Product Sales Revenue"},
            ],
            "nddc_levy": [
                {"debit_account": "NDDC Levy Expense", "credit_account": "NDDC Payable"},
            ],
            "education_tax": [
                {"debit_account": "Education Tax Expense", "credit_account": "Education Tax Payable"},
            ],
            "ppt_provision": [
                {"debit_account": "Petroleum Profit Tax Expense", "credit_account": "PPT Payable"},
            ],
            "abandonment_provision": [
                {"debit_account": "Decommissioning Expense", "credit_account": "Decommissioning Provision"},
            ],
        }

        if txn_type in posting_map:
            suggestions = posting_map[txn_type]
        else:
            suggestions = [
                {"debit_account": "Operating Expense", "credit_account": "Bank / Payable"},
            ]

        return suggestions

    def get_industry_kpis(self, financial_data: dict[str, Any]) -> dict[str, Decimal]:
        """Compute oil and gas specific KPIs.

        Args:
            financial_data: Dict with 'additions_to_reserves', 'production',
                           'finding_cost_total', 'opex_total', 'production_volume',
                           'realized_price', 'unit_cost', 'capex', 'proven_reserves',
                           'annual_production'.

        Returns:
            Dict of KPI name to computed value.
        """
        additions = _d(financial_data.get("additions_to_reserves", 0))
        production = _d(financial_data.get("production", 0))
        finding_cost_total = _d(financial_data.get("finding_cost_total", 0))
        opex_total = _d(financial_data.get("opex_total", 0))
        production_volume = _d(financial_data.get("production_volume", 1))
        realized_price = _d(financial_data.get("realized_price", 0))
        unit_cost = _d(financial_data.get("unit_cost", 0))
        capex = _d(financial_data.get("capex", 0))
        proven_reserves = _d(financial_data.get("proven_reserves", 1))
        annual_production = _d(financial_data.get("annual_production", 1))

        reserve_replacement = (additions / production * Decimal("100")).quantize(TWO_PLACES) if production > 0 else Decimal("0")
        finding_cost = (finding_cost_total / additions).quantize(TWO_PLACES) if additions > 0 else Decimal("0")
        lifting_cost = (opex_total / production_volume).quantize(TWO_PLACES) if production_volume > 0 else Decimal("0")
        netback = (realized_price - unit_cost).quantize(TWO_PLACES)
        reserve_life = (proven_reserves / annual_production).quantize(TWO_PLACES) if annual_production > 0 else Decimal("0")
        fdc = ((finding_cost_total + capex) / additions).quantize(TWO_PLACES) if additions > 0 else Decimal("0")

        return {
            "reserve_replacement_ratio_pct": reserve_replacement,
            "finding_cost_usd_per_boe": finding_cost,
            "lifting_cost_usd_per_boe": lifting_cost,
            "netback_usd_per_boe": netback,
            "reserve_life_index_years": reserve_life,
            "finding_development_cost": fdc,
        }

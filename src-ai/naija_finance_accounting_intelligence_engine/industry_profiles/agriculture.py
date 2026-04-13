"""Agriculture industry profile.

Author: Quadri Atharu

Covers Nigerian agriculture with crop/livestock revenue, IAS 41
biological asset inventory, CIT exemption for primary agriculture,
and key agricultural KPIs.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class AgricultureProfile:
    name: str = "Agriculture"
    typical_revenue_models: dict[str, str] = field(default_factory=lambda: {
        "crop_sales": "Revenue from sale of harvested crops",
        "livestock_sales": "Revenue from sale of livestock and livestock products",
        "produce_processing": "Revenue from primary processing of farm produce",
        "government_subsidy": "Government agricultural incentive/subsidy income",
    })
    cost_structures: dict[str, str] = field(default_factory=lambda: {
        "inputs": "Seeds, fertilizers, agrochemicals, feed",
        "labor": "Farm labor and seasonal workers",
        "irrigation_utilities": "Irrigation, water, electricity",
        "machinery_fuel": "Tractor operations, fuel, equipment maintenance",
        "biological_change": "Change in fair value of biological assets (IAS 41)",
    })
    tax_implications: dict[str, Decimal] = field(default_factory=lambda: {
        "cit_exempt_primary": Decimal("0"),
        "vat_exempt": Decimal("0"),
        "wht_contract": Decimal("5"),
    })
    compliance_requirements: list[str] = field(default_factory=lambda: [
        "CAC company registration",
        "NAQS phytosanitary certification",
        "SON product standards for exports",
        "NAFDAC registration (processed foods)",
        "Agricultural credit scheme compliance (CBN ACSF)",
        "Land Use Act compliance",
    ])
    inventory_logic: str = "Biological assets measured at fair value less cost to sell (IAS 41). Harvested produce at lower of cost and NRV (IAS 2)."
    depreciation_patterns: dict[str, str] = field(default_factory=lambda: {
        "farm_buildings": "Straight-line over 20-30 years",
        "machinery": "Straight-line over 5-10 years",
        "irrigation_systems": "Straight-line over 10-15 years",
    })
    typical_coa_ranges: dict[str, tuple[int, int]] = field(default_factory=lambda: {
        "biological_assets": (1100, 1199),
        "harvested_produce": (1200, 1299),
        "consumables": (1300, 1399),
        "land": (2100, 2199),
        "farm_machinery": (2200, 2299),
        "crop_revenue": (6100, 6199),
        "livestock_revenue": (6200, 6299),
    })
    key_kpis: list[str] = field(default_factory=lambda: [
        "yield_per_hectare",
        "cost_per_ton",
        "harvest_efficiency",
    ])

    def get_posting_suggestions(self, transaction: dict[str, Any]) -> list[dict[str, str]]:
        txn_type = transaction.get("type", "")
        posting_map: dict[str, list[dict[str, str]]] = {
            "crop_sale": [
                {"debit_account": "Bank / Receivables", "credit_account": "Crop Sales Revenue"},
            ],
            "livestock_sale": [
                {"debit_account": "Bank / Receivables", "credit_account": "Livestock Sales Revenue"},
            ],
            "input_purchase": [
                {"debit_account": "Consumables Inventory", "credit_account": "Bank / AP"},
            ],
            "labor_cost": [
                {"debit_account": "Farm Labor Expense", "credit_account": "Bank / Payroll Payable"},
            ],
            "biological_gain": [
                {"debit_account": "Biological Asset", "credit_account": "Gain on Biological Transformation"},
            ],
            "biological_loss": [
                {"debit_account": "Loss on Biological Transformation", "credit_account": "Biological Asset"},
            ],
            "harvest": [
                {"debit_account": "Harvested Produce Inventory", "credit_account": "Biological Asset / Gain on Harvest"},
            ],
        }
        return posting_map.get(txn_type, [{"debit_account": "Operating Expense", "credit_account": "Bank / AP"}])

    def get_industry_kpis(self, financial_data: dict[str, Any]) -> dict[str, Decimal]:
        total_yield = _d(financial_data.get("total_yield_kg", 0))
        hectares = _d(financial_data.get("hectares", 1))
        total_cost = _d(financial_data.get("total_cost", 0))
        output_tons = _d(financial_data.get("output_tons", 1))
        actual_harvest = _d(financial_data.get("actual_harvest", 0))
        expected_harvest = _d(financial_data.get("expected_harvest", 1))

        yield_per_ha = (total_yield / hectares).quantize(TWO_PLACES) if hectares > 0 else Decimal("0")
        cost_per_ton = (total_cost / output_tons).quantize(TWO_PLACES) if output_tons > 0 else Decimal("0")
        efficiency = (actual_harvest / expected_harvest * Decimal("100")).quantize(TWO_PLACES) if expected_harvest > 0 else Decimal("0")

        return {
            "yield_per_hectare_kg": yield_per_ha,
            "cost_per_ton": cost_per_ton,
            "harvest_efficiency_pct": efficiency,
        }

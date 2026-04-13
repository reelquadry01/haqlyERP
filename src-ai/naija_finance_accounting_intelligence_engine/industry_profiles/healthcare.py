"""Healthcare industry profile.

Author: Quadri Atharu

Covers Nigerian healthcare with consultation, procedure, pharmacy
revenue, CIT/VAT exemptions for non-profits, and key healthcare KPIs.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class HealthcareProfile:
    name: str = "Healthcare"
    typical_revenue_models: dict[str, str] = field(default_factory=lambda: {
        "consultations": "Revenue from medical consultations",
        "procedures_surgery": "Revenue from surgical and medical procedures",
        "pharmacy": "Revenue from pharmacy drug sales",
        "lab_diagnostics": "Revenue from laboratory and diagnostic services",
        "nhis_hmo": "Revenue from NHIS/HMO capitation and fee-for-service",
    })
    cost_structures: dict[str, str] = field(default_factory=lambda: {
        "drugs_pharmaceuticals": "Drug and pharmaceutical costs",
        "medical_supplies": "Medical consumables and supplies",
        "equipment_depreciation": "Medical equipment depreciation",
        "staff_costs": "Medical and administrative staff costs",
        "facility_costs": "Facility maintenance, utilities, cleaning",
    })
    tax_implications: dict[str, Decimal] = field(default_factory=lambda: {
        "cit_exempt_nonprofit": Decimal("0"),
        "vat_exempt_medical": Decimal("0"),
        "cit_for_profit": Decimal("30"),
    })
    compliance_requirements: list[str] = field(default_factory=lambda: [
        "FMC/MDCN registration",
        "NAFDAC drug registration",
        "NHIS accreditation (if applicable)",
        "PCN pharmacy registration",
        "MLSCN laboratory accreditation",
        "Radiographers registration",
    ])
    inventory_logic: str = "FIFO for drugs and pharmaceuticals. Expiry date tracking critical. Controlled substances tracked separately."
    depreciation_patterns: dict[str, str] = field(default_factory=lambda: {
        "medical_equipment": "Straight-line over 5-10 years",
        "buildings": "Straight-line over 25-50 years",
        "it_systems": "Straight-line over 3-5 years",
    })
    typical_coa_ranges: dict[str, tuple[int, int]] = field(default_factory=lambda: {
        "drug_inventory": (1100, 1199),
        "medical_supplies": (1200, 1299),
        "medical_equipment": (2100, 2199),
        "revenue": (6100, 6499),
        "drug_costs": (7100, 7199),
    })
    key_kpis: list[str] = field(default_factory=lambda: [
        "bed_occupancy",
        "patient_throughput",
        "drug_stockout_rate",
        "revenue_per_bed",
    ])

    def get_posting_suggestions(self, transaction: dict[str, Any]) -> list[dict[str, str]]:
        txn_type = transaction.get("type", "")
        posting_map: dict[str, list[dict[str, str]]] = {
            "consultation_fee": [
                {"debit_account": "Bank / Receivables", "credit_account": "Consultation Revenue"},
            ],
            "procedure_revenue": [
                {"debit_account": "Bank / Receivables", "credit_account": "Procedure Revenue"},
            ],
            "drug_sale": [
                {"debit_account": "Bank / Receivables", "credit_account": "Pharmacy Revenue"},
                {"debit_account": "Cost of Drugs Sold", "credit_account": "Drug Inventory"},
            ],
            "drug_purchase": [
                {"debit_account": "Drug Inventory", "credit_account": "Bank / AP"},
            ],
            "equipment_purchase": [
                {"debit_account": "Medical Equipment", "credit_account": "Bank / AP"},
            ],
            "staff_cost": [
                {"debit_account": "Staff Costs", "credit_account": "Bank / Payroll Payable"},
            ],
        }
        return posting_map.get(txn_type, [{"debit_account": "Operating Expense", "credit_account": "Bank / AP"}])

    def get_industry_kpis(self, financial_data: dict[str, Any]) -> dict[str, Decimal]:
        occupied_beds = _d(financial_data.get("occupied_beds", 0))
        total_beds = _d(financial_data.get("total_beds", 1))
        patients_treated = _d(financial_data.get("patients_treated", 0))
        stockout_incidents = _d(financial_data.get("stockout_incidents", 0))
        total_drug_lines = _d(financial_data.get("total_drug_lines", 1))
        total_revenue = _d(financial_data.get("total_revenue", 0))

        occupancy = (occupied_beds / total_beds * Decimal("100")).quantize(TWO_PLACES) if total_beds > 0 else Decimal("0")
        throughput = patients_treated
        stockout = (stockout_incidents / total_drug_lines * Decimal("100")).quantize(TWO_PLACES) if total_drug_lines > 0 else Decimal("0")
        rev_per_bed = (total_revenue / total_beds).quantize(TWO_PLACES) if total_beds > 0 else Decimal("0")

        return {
            "bed_occupancy_pct": occupancy,
            "patient_throughput": throughput,
            "drug_stockout_rate_pct": stockout,
            "revenue_per_bed": rev_per_bed,
        }

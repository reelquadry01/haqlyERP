"""Education industry profile.

Author: Quadri Atharu

Covers Nigerian education with tuition, fees, grants revenue,
CIT/VAT exemptions for non-profit, and key education KPIs.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class EducationProfile:
    name: str = "Education"
    typical_revenue_models: dict[str, str] = field(default_factory=lambda: {
        "tuition_fees": "Revenue from student tuition fees",
        "other_fees": "Revenue from registration, exam, and facility fees",
        "grants_subventions": "Government grants and TETFund allocations",
        "consultancy_services": "Revenue from consultancy and research services",
    })
    cost_structures: dict[str, str] = field(default_factory=lambda: {
        "salaries_wages": "Academic and non-academic staff costs",
        "facilities_maintenance": "Campus and facility maintenance",
        "teaching_materials": "Books, lab equipment, teaching aids",
        "utilities": "Power, water, internet connectivity",
    })
    tax_implications: dict[str, Decimal] = field(default_factory=lambda: {
        "cit_exempt_nonprofit": Decimal("0"),
        "vat_exempt_education": Decimal("0"),
        "cit_for_profit": Decimal("30"),
    })
    compliance_requirements: list[str] = field(default_factory=lambda: [
        "NUC/NCCE/NBTE accreditation",
        "TETFund compliance",
        "JAMB/POST-UME regulations",
        "NYSC mobilization compliance",
        "State Ministry of Education registration",
    ])
    inventory_logic: str = "FIFO for supplies and teaching materials. Book inventory at cost."
    depreciation_patterns: dict[str, str] = field(default_factory=lambda: {
        "buildings": "Straight-line over 30-50 years",
        "equipment": "Straight-line over 5-10 years",
        "it_infrastructure": "Straight-line over 3-5 years",
    })
    typical_coa_ranges: dict[str, tuple[int, int]] = field(default_factory=lambda: {
        "supplies": (1100, 1199),
        "buildings": (2100, 2199),
        "tuition_revenue": (6100, 6199),
        "fees_revenue": (6200, 6299),
        "staff_costs": (7100, 7299),
    })
    key_kpis: list[str] = field(default_factory=lambda: [
        "student_teacher_ratio",
        "graduation_rate",
        "revenue_per_student",
    ])

    def get_posting_suggestions(self, transaction: dict[str, Any]) -> list[dict[str, str]]:
        txn_type = transaction.get("type", "")
        posting_map: dict[str, list[dict[str, str]]] = {
            "tuition_received": [
                {"debit_account": "Bank", "credit_account": "Tuition Fee Revenue"},
            ],
            "fee_received": [
                {"debit_account": "Bank", "credit_account": "Other Fees Revenue"},
            ],
            "grant_received": [
                {"debit_account": "Bank", "credit_account": "Grant / Subvention Income"},
            ],
            "salary_payment": [
                {"debit_account": "Staff Costs", "credit_account": "Bank / Payroll Payable"},
            ],
            "material_purchase": [
                {"debit_account": "Teaching Materials Inventory", "credit_account": "Bank / AP"},
            ],
            "facility_maintenance": [
                {"debit_account": "Facility Maintenance Expense", "credit_account": "Bank / AP"},
            ],
        }
        return posting_map.get(txn_type, [{"debit_account": "Operating Expense", "credit_account": "Bank / AP"}])

    def get_industry_kpis(self, financial_data: dict[str, Any]) -> dict[str, Decimal]:
        total_students = _d(financial_data.get("total_students", 1))
        total_teachers = _d(financial_data.get("total_teachers", 1))
        graduated = _d(financial_data.get("graduated", 0))
        enrolled = _d(financial_data.get("enrolled", 1))
        total_revenue = _d(financial_data.get("total_revenue", 0))

        ratio = (total_students / total_teachers).quantize(TWO_PLACES) if total_teachers > 0 else Decimal("0")
        grad_rate = (graduated / enrolled * Decimal("100")).quantize(TWO_PLACES) if enrolled > 0 else Decimal("0")
        rev_student = (total_revenue / total_students).quantize(TWO_PLACES) if total_students > 0 else Decimal("0")

        return {
            "student_teacher_ratio": ratio,
            "graduation_rate_pct": grad_rate,
            "revenue_per_student": rev_student,
        }

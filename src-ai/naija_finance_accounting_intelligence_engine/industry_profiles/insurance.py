"""Insurance industry profile.

Author: Quadri Atharu

Covers Nigerian insurance with premium income, claims, reinsurance,
solvency margins, and key insurance KPIs per NAICOM regulations.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class InsuranceProfile:
    name: str = "Insurance"
    typical_revenue_models: dict[str, str] = field(default_factory=lambda: {
        "premium_income": "Income from insurance policy premiums",
        "investment_income": "Income from investment of premium reserves",
        "reinsurance_commission": "Commission earned from reinsurance treaties",
        "fee_income": "Policy administration and endorsement fees",
    })
    cost_structures: dict[str, str] = field(default_factory=lambda: {
        "claims_paid": "Net claims paid to policyholders",
        "reinsurance_premium": "Premium ceded to reinsurers",
        "acquisition_cost": "Commission to brokers and agents",
        "operating_expenses": "Administration, IT, and management costs",
        "change_in_provision": "Change in unexpired risk and outstanding claims provisions",
    })
    tax_implications: dict[str, Decimal] = field(default_factory=lambda: {
        "cit": Decimal("30"),
        "vat_on_fees": Decimal("7.5"),
        "education_tax": Decimal("2"),
    })
    compliance_requirements: list[str] = field(default_factory=lambda: [
        "NAICOM regulatory compliance",
        "Solvency margin requirements",
        "Minimum capital base (N10B life, N5B general)",
        "Annual statutory returns to NAICOM",
        "IFRS 17 insurance contract accounting",
        "FIRS tax returns",
        "Risk-based supervision requirements",
    ])
    inventory_logic: str = "N/A - Insurance companies do not carry physical inventory"
    depreciation_patterns: dict[str, str] = field(default_factory=lambda: {
        "office_buildings": "Straight-line over 25-50 years",
        "it_systems": "Straight-line over 3-5 years",
        "furniture": "Straight-line over 5-10 years",
    })
    typical_coa_ranges: dict[str, tuple[int, int]] = field(default_factory=lambda: {
        "investments": (1100, 1499),
        "receivables": (1500, 1699),
        "provisions": (2100, 2299),
        "premium_income": (4100, 4299),
        "claims_expense": (5100, 5299),
        "reinsurance": (5300, 5499),
    })
    key_kpis: list[str] = field(default_factory=lambda: [
        "loss_ratio",
        "combined_ratio",
        "solvency_margin",
        "retention_ratio",
    ])

    def get_posting_suggestions(self, transaction: dict[str, Any]) -> list[dict[str, str]]:
        txn_type = transaction.get("type", "")
        posting_map: dict[str, list[dict[str, str]]] = {
            "premium_received": [
                {"debit_account": "Bank", "credit_account": "Premium Income"},
            ],
            "claim_paid": [
                {"debit_account": "Claims Expense", "credit_account": "Bank"},
            ],
            "reinsurance_premium": [
                {"debit_account": "Reinsurance Premium Ceded", "credit_account": "Bank / Reinsurance Payable"},
            ],
            "reinsurance_recovery": [
                {"debit_account": "Bank / Reinsurance Receivable", "credit_account": "Reinsurance Recovery Income"},
            ],
            "commission_paid": [
                {"debit_account": "Acquisition Cost - Commission", "credit_account": "Bank / Commission Payable"},
            ],
            "provision_increase": [
                {"debit_account": "Change in Provision Expense", "credit_account": "Unexpired Risk Provision / Outstanding Claims"},
            ],
            "investment_income": [
                {"debit_account": "Bank / Interest Receivable", "credit_account": "Investment Income"},
            ],
        }
        return posting_map.get(txn_type, [{"debit_account": "Operating Expense", "credit_account": "Bank / AP"}])

    def get_industry_kpis(self, financial_data: dict[str, Any]) -> dict[str, Decimal]:
        claims_incurred = _d(financial_data.get("claims_incurred", 0))
        premium_earned = _d(financial_data.get("premium_earned", 1))
        operating_expenses = _d(financial_data.get("operating_expenses", 0))
        acquisition_cost = _d(financial_data.get("acquisition_cost", 0))
        available_capital = _d(financial_data.get("available_capital", 0))
        minimum_capital = _d(financial_data.get("minimum_capital", 1))
        premium_retained = _d(financial_data.get("premium_retained", 0))
        gross_premium = _d(financial_data.get("gross_premium", 1))

        loss_ratio = (claims_incurred / premium_earned * Decimal("100")).quantize(TWO_PLACES) if premium_earned > 0 else Decimal("0")
        combined_ratio = ((claims_incurred + operating_expenses + acquisition_cost) / premium_earned * Decimal("100")).quantize(TWO_PLACES) if premium_earned > 0 else Decimal("0")
        solvency = (available_capital / minimum_capital * Decimal("100")).quantize(TWO_PLACES) if minimum_capital > 0 else Decimal("0")
        retention = (premium_retained / gross_premium * Decimal("100")).quantize(TWO_PLACES) if gross_premium > 0 else Decimal("0")

        return {
            "loss_ratio_pct": loss_ratio,
            "combined_ratio_pct": combined_ratio,
            "solvency_margin_pct": solvency,
            "retention_ratio_pct": retention,
        }

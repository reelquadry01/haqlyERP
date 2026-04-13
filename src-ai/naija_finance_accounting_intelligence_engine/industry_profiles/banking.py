"""Banking industry profile.

Author: Quadri Atharu

Covers Nigerian banking with interest income, fees, commissions,
loan loss provisions, CBN regulatory requirements, NPL ratios,
and key banking KPIs.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class BankingProfile:
    name: str = "Banking"
    typical_revenue_models: dict[str, str] = field(default_factory=lambda: {
        "interest_income": "Income from loans, advances, and money market placement",
        "fee_commission_income": "Income from account maintenance, transfers, LC fees",
        "treasury_income": "Income from trading securities and FX",
        "e_banking_income": "Income from digital banking services",
    })
    cost_structures: dict[str, str] = field(default_factory=lambda: {
        "interest_expense": "Cost of customer deposits and borrowings",
        "loan_loss_provision": "IFRS 9 ECL provision for credit losses",
        "personnel_cost": "Staff salaries, training, and benefits",
        "it_infrastructure": "Core banking, IT systems, cybersecurity",
        "regulatory_costs": "CBN fees, NDIC premium, AMCON levy",
    })
    tax_implications: dict[str, Decimal] = field(default_factory=lambda: {
        "cit": Decimal("30"),
        "vat_on_fees": Decimal("7.5"),
        "wht_commission": Decimal("10"),
        "education_tax": Decimal("2"),
    })
    compliance_requirements: list[str] = field(default_factory=lambda: [
        "CBN Prudential Guidelines compliance",
        "NDIC deposit insurance",
        "AMCON levy payment",
        "Basel III capital adequacy",
        "TSA compliance for government banking",
        "CBN AML/CFT regulations",
        "FIRS tax returns",
        "NBS statistical returns",
    ])
    inventory_logic: str = "N/A - Banks do not carry physical inventory"
    depreciation_patterns: dict[str, str] = field(default_factory=lambda: {
        "it_equipment": "Straight-line over 3-5 years",
        "office_buildings": "Straight-line over 25-50 years",
        "furniture_fixtures": "Straight-line over 5-10 years",
    })
    typical_coa_ranges: dict[str, tuple[int, int]] = field(default_factory=lambda: {
        "cash_balances": (1000, 1099),
        "loans_advances": (1100, 1399),
        "investment_securities": (1400, 1599),
        "customer_deposits": (2100, 2399),
        "borrowings": (2400, 2599),
        "interest_income": (4100, 4199),
        "interest_expense": (5100, 5199),
        "fee_commission_income": (4200, 4299),
        "loan_loss_provision": (5300, 5399),
    })
    key_kpis: list[str] = field(default_factory=lambda: [
        "loan_to_deposit_ratio",
        "npl_ratio",
        "cost_to_income_ratio",
        "net_interest_margin",
    ])

    def get_posting_suggestions(self, transaction: dict[str, Any]) -> list[dict[str, str]]:
        txn_type = transaction.get("type", "")
        posting_map: dict[str, list[dict[str, str]]] = {
            "loan_disbursement": [
                {"debit_account": "Loans & Advances", "credit_account": "Customer Deposit Account"},
            ],
            "interest_income": [
                {"debit_account": "Customer Deposit Account", "credit_account": "Interest Income"},
            ],
            "interest_expense": [
                {"debit_account": "Interest Expense", "credit_account": "Customer Deposit Account"},
            ],
            "fee_income": [
                {"debit_account": "Customer Deposit Account", "credit_account": "Fee & Commission Income"},
            ],
            "loan_loss_provision": [
                {"debit_account": "Loan Loss Provision Expense", "credit_account": "ECL Provision (IFRS 9)"},
            ],
            "deposit_receipt": [
                {"debit_account": "Bank / Cash", "credit_account": "Customer Deposit Account"},
            ],
            "withdrawal": [
                {"debit_account": "Customer Deposit Account", "credit_account": "Bank / Cash"},
            ],
        }
        return posting_map.get(txn_type, [{"debit_account": "Operating Expense", "credit_account": "Bank / Payable"}])

    def get_industry_kpis(self, financial_data: dict[str, Any]) -> dict[str, Decimal]:
        total_loans = _d(financial_data.get("total_loans", 0))
        total_deposits = _d(financial_data.get("total_deposits", 1))
        npl_balance = _d(financial_data.get("npl_balance", 0))
        gross_loans = _d(financial_data.get("gross_loans", 1))
        operating_income = _d(financial_data.get("operating_income", 0))
        operating_expense = _d(financial_data.get("operating_expense", 1))
        interest_income = _d(financial_data.get("interest_income", 0))
        interest_expense = _d(financial_data.get("interest_expense", 0))
        earning_assets = _d(financial_data.get("earning_assets", 1))

        ltd = (total_loans / total_deposits * Decimal("100")).quantize(TWO_PLACES) if total_deposits > 0 else Decimal("0")
        npl_ratio = (npl_balance / gross_loans * Decimal("100")).quantize(TWO_PLACES) if gross_loans > 0 else Decimal("0")
        cti = (operating_expense / operating_income * Decimal("100")).quantize(TWO_PLACES) if operating_income > 0 else Decimal("0")
        nim = ((interest_income - interest_expense) / earning_assets * Decimal("100")).quantize(TWO_PLACES) if earning_assets > 0 else Decimal("0")

        return {
            "loan_to_deposit_ratio_pct": ltd,
            "npl_ratio_pct": npl_ratio,
            "cost_to_income_ratio_pct": cti,
            "net_interest_margin_pct": nim,
        }

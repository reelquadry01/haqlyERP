"""Nigerian tax law reference and rate tables.

Author: Quadri Atharu

Provides current tax rates, exemptions, and filing requirements for
all Nigerian tax types including CIT, VAT, WHT, PPT, CGT, Education
Tax, Stamp Duty, and minimum tax provisions.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class TaxRateInfo:
    tax_type: str
    rates: dict[str, Decimal]
    effective_date: str
    authority: str


@dataclass
class ExemptionInfo:
    tax_type: str
    exemptions: list[str]
    conditions: list[str]


@dataclass
class FilingInfo:
    tax_type: str
    filing_frequency: str
    due_date_rule: str
    penalty_rate: Decimal
    required_documents: list[str]


CURRENT_TAX_RATES: dict[str, TaxRateInfo] = {
    "CIT": TaxRateInfo(
        tax_type="Companies Income Tax",
        rates={
            "small_company": Decimal("0"),
            "medium": Decimal("20"),
            "large": Decimal("30"),
        },
        effective_date="2020-01-01",
        authority="FIRS",
    ),
    "VAT": TaxRateInfo(
        tax_type="Value Added Tax",
        rates={
            "standard": Decimal("7.5"),
            "zero_rated": Decimal("0"),
            "exempt": Decimal("0"),
        },
        effective_date="2020-02-01",
        authority="FIRS",
    ),
    "WHT": TaxRateInfo(
        tax_type="Withholding Tax",
        rates={
            "dividends": Decimal("10"),
            "interest": Decimal("10"),
            "rent": Decimal("10"),
            "royalties": Decimal("5"),
            "management_fees": Decimal("10"),
            "consultancy": Decimal("5"),
            "contract": Decimal("5"),
            "commission": Decimal("5"),
            "construction": Decimal("2.5"),
        },
        effective_date="2022-01-01",
        authority="FIRS",
    ),
    "PPT": TaxRateInfo(
        tax_type="Petroleum Profit Tax",
        rates={
            "joint_venture": Decimal("50"),
            "production_sharing_contract": Decimal("85"),
        },
        effective_date="2020-01-01",
        authority="FIRS",
    ),
    "CGT": TaxRateInfo(
        tax_type="Capital Gains Tax",
        rates={
            "standard": Decimal("10"),
        },
        effective_date="2020-01-01",
        authority="FIRS",
    ),
    "Education_Tax": TaxRateInfo(
        tax_type="Education Tax",
        rates={
            "standard": Decimal("2"),
        },
        effective_date="2022-01-01",
        authority="FIRS",
    ),
    "Stamp_Duty": TaxRateInfo(
        tax_type="Stamp Duty",
        rates={
            "share_capital": Decimal("0.375"),
            "electronic_transfer_above_10k": Decimal("50"),
            "receipt_above_10k": Decimal("50"),
        },
        effective_date="2020-01-01",
        authority="FIRS",
    ),
    "Minimum_Tax": TaxRateInfo(
        tax_type="Minimum Tax",
        rates={
            "pct_of_turnover": Decimal("0.5"),
            "pct_of_gross_profit": Decimal("0.5"),
        },
        effective_date="2020-01-01",
        authority="FIRS",
    ),
}

TAX_EXEMPTIONS: dict[str, ExemptionInfo] = {
    "CIT": ExemptionInfo(
        tax_type="CIT",
        exemptions=[
            "Small companies (turnover < NGN 25M)",
            "Pioneer status companies (5-year tax holiday, renewable for 2 years)",
            "Agricultural companies (primary production)",
            "Non-profit organizations (registered)",
            "Export incentives (profit from export of manufactured goods)",
        ],
        conditions=[
            "Must file returns even if exempt",
            "Pioneer status requires NIPC certificate",
            "Agricultural exemption limited to primary production",
        ],
    ),
    "VAT": ExemptionInfo(
        tax_type="VAT",
        exemptions=[
            "Medical services and pharmaceuticals",
            "Educational services and materials",
            "Financial services (loans, interest)",
            "Agricultural products and inputs",
            "Baby products",
            "Commercial aircraft and parts",
            "LPG, diesel, and petroleum products",
        ],
        conditions=[
            "Zero-rated items (exports) differ from exempt items",
            "Input VAT not recoverable on exempt supplies",
        ],
    ),
    "CGT": ExemptionInfo(
        tax_type="CGT",
        exemptions=[
            "Gains from disposal of principal private residence",
            "Gains from life insurance policies",
            "Gains from Nigerian government securities",
            "Severance pay and compensation for loss of office",
        ],
        conditions=[
            "Principal private residence exemption applies once",
            "Must elect within 1 year of disposal",
        ],
    ),
}

FILING_REQUIREMENTS: dict[str, FilingInfo] = {
    "CIT": FilingInfo(
        tax_type="CIT",
        filing_frequency="Annual",
        due_date_rule="Due 6 months after financial year-end; installment payments due by 6th month",
        penalty_rate=Decimal("10"),
        required_documents=[
            "Self-assessment return",
            "Audited financial statements",
            "Tax computation schedule",
            "Withholding tax credit notes",
            "Capital allowance schedule",
        ],
    ),
    "VAT": FilingInfo(
        tax_type="VAT",
        filing_frequency="Monthly",
        due_date_rule="Due by 21st of the following month",
        penalty_rate=Decimal("5"),
        required_documents=[
            "VAT return form",
            "VAT schedule (output and input)",
            "Supporting invoices",
        ],
    ),
    "WHT": FilingInfo(
        tax_type="WHT",
        filing_frequency="Monthly",
        due_date_rule="Due by 21st of the following month",
        penalty_rate=Decimal("10"),
        required_documents=[
            "WHT return form",
            "WHT schedule",
            "Credit notes issued",
        ],
    ),
    "PPT": FilingInfo(
        tax_type="PPT",
        filing_frequency="Monthly (installment) / Annual (assessment)",
        due_date_rule="Monthly installments; annual return within 5 months of year-end",
        penalty_rate=Decimal("10"),
        required_documents=[
            "PPT return",
            "Oil and gas financial statements",
            "Cost oil and profit oil computation",
        ],
    ),
    "PAYE": FilingInfo(
        tax_type="PAYE",
        filing_frequency="Monthly",
        due_date_rule="Due by 10th of the following month",
        penalty_rate=Decimal("10"),
        required_documents=[
            "PAYE schedule",
            "Employee tax deduction cards",
            "Pension deduction schedule",
        ],
    ),
}


class TaxLawReference:
    """Reference class for Nigerian tax law rates, exemptions, and filing requirements."""

    def get_current_rates(self, tax_type: str) -> TaxRateInfo:
        """Get current tax rates for a given tax type.

        Args:
            tax_type: Tax type code (CIT, VAT, WHT, PPT, CGT, Education_Tax, Stamp_Duty, Minimum_Tax).

        Returns:
            TaxRateInfo with current rates.

        Raises:
            ValueError: If tax_type is not recognized.
        """
        if tax_type not in CURRENT_TAX_RATES:
            raise ValueError(f"Tax type '{tax_type}' not recognized. Available: {list(CURRENT_TAX_RATES.keys())}")
        return CURRENT_TAX_RATES[tax_type]

    def get_exemptions(self, tax_type: str) -> ExemptionInfo:
        """Get exemption list for a given tax type.

        Args:
            tax_type: Tax type code.

        Returns:
            ExemptionInfo with exemptions and conditions.

        Raises:
            ValueError: If tax_type has no exemptions on record.
        """
        if tax_type not in TAX_EXEMPTIONS:
            raise ValueError(f"No exemptions on record for '{tax_type}'. Available: {list(TAX_EXEMPTIONS.keys())}")
        return TAX_EXEMPTIONS[tax_type]

    def get_filing_requirements(self, tax_type: str) -> FilingInfo:
        """Get filing requirements for a given tax type.

        Args:
            tax_type: Tax type code.

        Returns:
            FilingInfo with frequency, due dates, and required documents.

        Raises:
            ValueError: If tax_type is not recognized.
        """
        if tax_type not in FILING_REQUIREMENTS:
            raise ValueError(f"Filing requirements not available for '{tax_type}'. Available: {list(FILING_REQUIREMENTS.keys())}")
        return FILING_REQUIREMENTS[tax_type]

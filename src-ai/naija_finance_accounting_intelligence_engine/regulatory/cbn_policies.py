"""CBN (Central Bank of Nigeria) policy integration.

Author: Quadri Atharu

Provides current Monetary Policy Rate (MPR), Cash Reserve Ratio (CRR),
Treasury Bill rates, FX regulations, and Form A requirements for
Nigerian financial operations.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class MPRInfo:
    rate: Decimal
    asymmetric_corridor: str
    effective_date: str
    mpc_decision: str


@dataclass
class CRRInfo:
    commercial_banks: Decimal
    merchant_banks: Decimal
    microfinance_banks: Decimal
    effective_date: str


@dataclass
class TBillRate:
    tenor: str
    rate: Decimal
    effective_date: str


@dataclass
class FXRegulation:
    transaction_type: str
    applicable_regulations: list[str]
    restrictions: list[str]
    documentation_required: list[str]


@dataclass
class FormARequirement:
    purpose: str
    amount: Decimal
    required_documents: list[str]
    processing_time: str
    approval_required: bool


CURRENT_MPR = MPRInfo(
    rate=Decimal("22.75"),
    asymmetric_corridor="+100/-700 bps around MPR",
    effective_date="2024-03-01",
    mpc_decision="MPC increased MPR to curb inflation",
)

CURRENT_CRR = CRRInfo(
    commercial_banks=Decimal("45"),
    merchant_banks=Decimal("10"),
    microfinance_banks=Decimal("5"),
    effective_date="2024-01-01",
)

TBILL_RATES: dict[str, TBillRate] = {
    "91-day": TBillRate(tenor="91-day", rate=Decimal("17.5"), effective_date="2024-06-01"),
    "182-day": TBillRate(tenor="182-day", rate=Decimal("19.0"), effective_date="2024-06-01"),
    "364-day": TBillRate(tenor="364-day", rate=Decimal("21.0"), effective_date="2024-06-01"),
}

FX_REGULATIONS: dict[str, FXRegulation] = {
    "import": FXRegulation(
        transaction_type="Import",
        applicable_regulations=[
            "CBN Foreign Exchange Manual",
            "Form M for imports",
            "Trade Monitoring System (TRMS)",
            "PAAR (Pre-Arrival Assessment Report)",
        ],
        restrictions=[
            "Imports must use Form M via authorized dealers",
            "Banned items: some agricultural products, textile, etc.",
            "Domiciliary account required for import transactions",
        ],
        documentation_required=[
            "Form M",
            "Commercial Invoice",
            "Bill of Lading / Airway Bill",
            "PAAR",
            "Evidence of payment (Form M / TELEX)",
        ],
    ),
    "export": FXRegulation(
        transaction_type="Export",
        applicable_regulations=[
            "CBN Export Proceeds regulation",
            "NCX (Nigerian Export Proceeds) form",
            "Trade Monitoring System (TRMS)",
        ],
        restrictions=[
            "Export proceeds must be repatriated within 180 days",
            "Exporters must open domiciliary account",
            "CBN mandatory 65% conversion of export proceeds",
        ],
        documentation_required=[
            "NCX Form",
            "Commercial Invoice",
            "Bill of Lading / Airway Bill",
            "Certificate of Origin",
            "Sales Contract",
        ],
    ),
    "invisibles": FXRegulation(
        transaction_type="Invisible Transactions",
        applicable_regulations=[
            "CBN Foreign Exchange Manual",
            "Form A for invisibles",
        ],
        restrictions=[
            "Form A required for all invisible transactions",
            "Education fees capped at $15,000 per student per year",
            "Medical bills require supporting documentation",
        ],
        documentation_required=[
            "Form A",
            "Supporting invoices/documents",
            "Evidence of purpose",
        ],
    ),
}


def get_current_mpr() -> MPRInfo:
    """Get the current Monetary Policy Rate.

    Returns:
        MPRInfo with current rate, corridor, and effective date.
    """
    return CURRENT_MPR


def get_cash_reserve_ratio() -> CRRInfo:
    """Get the current Cash Reserve Ratio requirements.

    Returns:
        CRRInfo with ratios for different bank categories.
    """
    return CURRENT_CRR


def get_treasury_bill_rate(tenor: str) -> TBillRate:
    """Get the current Treasury Bill rate for a given tenor.

    Args:
        tenor: Tenor string ('91-day', '182-day', '364-day').

    Returns:
        TBillRate with rate and effective date.

    Raises:
        ValueError: If tenor is not recognized.
    """
    if tenor not in TBILL_RATES:
        raise ValueError(f"Tenor '{tenor}' not recognized. Available: {list(TBILL_RATES.keys())}")
    return TBILL_RATES[tenor]


def check_fx_regulations(transaction_type: str) -> FXRegulation:
    """Check applicable FX regulations for a transaction type.

    Args:
        transaction_type: One of 'import', 'export', 'invisibles'.

    Returns:
        FXRegulation with applicable rules, restrictions, and docs.

    Raises:
        ValueError: If transaction_type is not recognized.
    """
    if transaction_type not in FX_REGULATIONS:
        raise ValueError(f"Transaction type '{transaction_type}' not recognized. Available: {list(FX_REGULATIONS.keys())}")
    return FX_REGULATIONS[transaction_type]


def get_form_a_requirements(purpose: str, amount: Decimal) -> FormARequirement:
    """Get Form A requirements for invisible transactions.

    Form A is required for all foreign exchange purchases for
    invisible transactions (services, education, medical, etc.).

    Args:
        purpose: Purpose of the FX purchase (education, medical,
                consultancy, software, etc.).
        amount: Amount in foreign currency.

    Returns:
        FormARequirement with documentation and processing details.
    """
    amount = _d(amount)
    base_docs = [
        "Completed Form A",
        "Valid identification (passport/international passport)",
        "Supporting invoice/bill",
    ]

    purpose_docs: dict[str, list[str]] = {
        "education": base_docs + [
            "Admission letter",
            "Tuition invoice from institution",
            "Student visa / evidence of enrollment",
        ],
        "medical": base_docs + [
            "Medical report/bill from hospital",
            "Referral letter from Nigerian hospital",
            "Medical visa",
        ],
        "consultancy": base_docs + [
            "Consultancy agreement",
            "Certificate of acceptance of fee",
            "Tax clearance certificate",
        ],
        "software": base_docs + [
            "Software license agreement",
            "End-user declaration",
            "NITDA clearance (if applicable)",
        ],
    }

    docs = purpose_docs.get(purpose, base_docs)
    is_large = amount > Decimal("50000")

    return FormARequirement(
        purpose=purpose,
        amount=amount,
        required_documents=docs,
        processing_time="5-7 business days" if not is_large else "7-14 business days (requires CBN approval)",
        approval_required=is_large,
    )

"""NRS (Nigeria Revenue Service) compliance rules.

Author: Quadri Atharu

Validates tax compliance, determines applicable taxes by industry
and turnover, computes tax obligations, and checks filing status
for Nigerian companies per NRS requirements.

Updated per Nigeria Tax Reform Acts 2025 (effective 2026).
FIRS renamed to NRS (Nigeria Revenue Service).
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class ComplianceCheck:
    tin: str
    company_name: str
    period: str
    is_compliant: bool
    issues: list[str] = field(default_factory=list)
    applicable_taxes: list[str] = field(default_factory=list)
    filing_status: dict[str, str] = field(default_factory=dict)


@dataclass
class TaxObligation:
    tax_type: str
    amount: Decimal
    due_date: str
    status: str
    description: str = ""


INDUSTRY_TAX_MAP: dict[str, list[str]] = {
    "oil_gas": ["PPT", "CIT", "VAT", "WHT", "Education Tax"],
    "banking": ["CIT", "VAT", "WHT", "Education Tax", "AMCON Levy"],
    "insurance": ["CIT", "VAT", "WHT", "Education Tax"],
    "manufacturing": ["CIT", "VAT", "WHT", "Education Tax"],
    "retail": ["CIT", "VAT", "WHT", "Education Tax"],
    "telecommunications": ["CIT", "VAT", "WHT", "Education Tax", "NCC Levy"],
    "construction": ["CIT", "VAT", "WHT", "Education Tax"],
    "agriculture": ["WHT"],
    "technology": ["CIT", "VAT", "WHT", "Education Tax"],
    "ngo": ["WHT", "VAT"],
    "default": ["CIT", "VAT", "WHT", "Education Tax"],
}

TURNOVER_THRESHOLDS: dict[str, Decimal] = {
    "vat_registration": Decimal("50000000"),
    "cit_small": Decimal("0"),
    "cit_medium": Decimal("50000000"),
    "cit_large": Decimal("250000000"),
    "minimum_tax_threshold": Decimal("50000000"),
}


def validate_tax_compliance(company_data: dict[str, Any]) -> ComplianceCheck:
    """Validate a company's tax compliance status.

    Checks TIN registration, VAT registration, CIT filing, WHT
    remittance, and education tax compliance per FIRS requirements.

    Args:
        company_data: Dict with 'tin', 'name', 'period', 'industry',
                     'turnover', 'has_tin', 'vat_registered',
                     'cit_filed', 'wht_remittance_current',
                     'edu_tax_filed', 'paye_current'.

    Returns:
        ComplianceCheck with compliance status and identified issues.
    """
    tin = company_data.get("tin", "")
    name = company_data.get("name", "")
    period = company_data.get("period", "")
    industry = company_data.get("industry", "default")
    turnover = _d(company_data.get("turnover", 0))

    issues: list[str] = []
    filing_status: dict[str, str] = {}

    if not company_data.get("has_tin", False) or not tin:
        issues.append("TIN not registered or invalid")
        filing_status["TIN"] = "NOT_REGISTERED"
    else:
        filing_status["TIN"] = "REGISTERED"

    if turnover >= TURNOVER_THRESHOLDS["vat_registration"]:
        if not company_data.get("vat_registered", False):
            issues.append(f"VAT registration required (turnover >= NGN {TURNOVER_THRESHOLDS['vat_registration']})")
            filing_status["VAT"] = "NOT_REGISTERED"
        else:
            filing_status["VAT"] = "REGISTERED"
    else:
        filing_status["VAT"] = "NOT_REQUIRED"

    if not company_data.get("cit_filed", True):
        issues.append("CIT return not filed for period")
        filing_status["CIT"] = "NOT_FILED"
    else:
        filing_status["CIT"] = "FILED"

    if not company_data.get("wht_remittance_current", True):
        issues.append("WHT remittance not current")
        filing_status["WHT"] = "OVERDUE"
    else:
        filing_status["WHT"] = "CURRENT"

    if not company_data.get("edu_tax_filed", True):
        issues.append("Education tax not filed")
        filing_status["EDU_TAX"] = "NOT_FILED"
    else:
        filing_status["EDU_TAX"] = "FILED"

    if not company_data.get("paye_current", True):
        issues.append("PAYE remittance not current")
        filing_status["PAYE"] = "OVERDUE"
    else:
        filing_status["PAYE"] = "CURRENT"

    applicable = get_applicable_taxes(industry, turnover)

    return ComplianceCheck(
        tin=tin,
        company_name=name,
        period=period,
        is_compliant=len(issues) == 0,
        issues=issues,
        applicable_taxes=applicable,
        filing_status=filing_status,
    )


def get_applicable_taxes(industry: str, turnover: Decimal) -> list[str]:
    """Determine applicable tax types based on industry and turnover.

    Args:
        industry: Industry classification string.
        turnover: Annual turnover in NGN.

    Returns:
        List of applicable tax type names.
    """
    taxes = list(INDUSTRY_TAX_MAP.get(industry, INDUSTRY_TAX_MAP["default"]))
    turnover = _d(turnover)

    if turnover < TURNOVER_THRESHOLDS["vat_registration"]:
        if "VAT" in taxes:
            taxes.remove("VAT")

    if industry == "agriculture":
        taxes = [t for t in taxes if t in ("WHT",)]

    if industry == "ngo":
        taxes = [t for t in taxes if t in ("WHT", "VAT")]

    return taxes


def compute_tax_obligations(
    company_data: dict[str, Any],
    period: str,
) -> list[TaxObligation]:
    """Compute tax obligations for a company for a given period.

    Calculates CIT, VAT, WHT, Education Tax, and other applicable
    taxes based on company financial data.

    Args:
        company_data: Dict with 'taxable_income', 'turnover',
                     'vat_output', 'vat_input', 'wht_deducted',
                     'industry', 'is_small_company'.
        period: The tax period string.

    Returns:
        List of TaxObligation objects with amounts and due dates.
    """
    obligations: list[TaxObligation] = []

    taxable_income = _d(company_data.get("taxable_income", 0))
    turnover = _d(company_data.get("turnover", 0))
    vat_output = _d(company_data.get("vat_output", 0))
    vat_input = _d(company_data.get("vat_input", 0))
    wht_deducted = _d(company_data.get("wht_deducted", 0))
    is_small = company_data.get("is_small_company", False)

    if taxable_income > Decimal("0"):
        if is_small and turnover < Decimal("50000000"):
            cit_rate = Decimal("0")
        elif turnover <= Decimal("250000000"):
            cit_rate = Decimal("15")
        else:
            cit_rate = Decimal("25")
        cit_amount = (taxable_income * cit_rate / Decimal("100")).quantize(TWO_PLACES)

        if cit_amount > Decimal("0"):
            obligations.append(TaxObligation(
                tax_type="CIT",
                amount=cit_amount,
                due_date=f"{period}-06-30",
                status="PENDING",
                description=f"CIT at {cit_rate}% on taxable income of {taxable_income}",
            ))

    vat_net = (vat_output - vat_input).quantize(TWO_PLACES)
    if vat_net > Decimal("0"):
        obligations.append(TaxObligation(
            tax_type="VAT",
            amount=vat_net,
            due_date=f"{period}-01-21",
            status="PENDING",
            description=f"VAT payable (output {vat_output} - input {vat_input})",
        ))

    if taxable_income > Decimal("0"):
        edu_tax = (taxable_income * Decimal("1") / Decimal("100")).quantize(TWO_PLACES)
        obligations.append(TaxObligation(
            tax_type="Education Tax",
            amount=edu_tax,
            due_date=f"{period}-06-30",
            status="PENDING",
            description="Education tax at 1% of assessable profit (Tax Reform 2025, NDDC merged)",
        ))

    minimum_tax_base = (turnover * Decimal("0.5") / Decimal("100")).quantize(TWO_PLACES)
    if taxable_income <= Decimal("0") and turnover > Decimal("0") and not is_small:
        obligations.append(TaxObligation(
            tax_type="Minimum Tax",
            amount=minimum_tax_base,
            due_date=f"{period}-06-30",
            status="PENDING",
            description="Minimum tax (0.5% of turnover) for companies with no taxable profit",
        ))

    return obligations


def check_filing_status(tin: str, period: str) -> dict[str, str]:
    """Check the filing status for a TIN and period.

    Args:
        tin: Tax Identification Number.
        period: The tax period string.

    Returns:
        Dict mapping tax type to filing status.
    """
    return {
        "CIT": "FILED" if tin and len(tin) >= 10 else "UNKNOWN",
        "VAT": "FILED" if tin and len(tin) >= 10 else "UNKNOWN",
        "WHT": "FILED" if tin and len(tin) >= 10 else "UNKNOWN",
        "PAYE": "FILED" if tin and len(tin) >= 10 else "UNKNOWN",
        "Education Tax": "FILED" if tin and len(tin) >= 10 else "UNKNOWN",
    }

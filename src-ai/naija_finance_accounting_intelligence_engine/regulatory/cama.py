"""CAMA (Companies and Allied Matters Act) compliance.

Author: Quadri Atharu

Implements CAMA 2020 requirements for company registration, annual
return filing, and director requirements for Nigerian companies.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any


@dataclass
class RegistrationRequirement:
    requirement: str
    description: str
    is_mandatory: bool
    applicable_to: list[str]


@dataclass
class AnnualReturnValidation:
    company_name: str
    is_valid: bool
    issues: list[str] = field(default_factory=list)
    required_documents: list[str] = field(default_factory=list)


@dataclass
class DirectorRequirement:
    company_type: str
    minimum_directors: int
    maximum_directors: int
    secretary_required: bool
    residency_requirement: str


COMPANY_TYPES = {
    "private_limited": {
        "name": "Private Limited Company (LTD)",
        "min_directors": 1,
        "max_directors": 50,
        "secretary_required": True,
        "min_share_capital": Decimal("100000"),
        "residency": "At least 1 Nigerian director (post-CAMA 2020: no residency requirement for private)",
    },
    "public_limited": {
        "name": "Public Limited Company (PLC)",
        "min_directors": 2,
        "max_directors": 50,
        "secretary_required": True,
        "min_share_capital": Decimal("2000000"),
        "residency": "At least 1 Nigerian director",
    },
    "company_limited_by_guarantee": {
        "name": "Company Limited by Guarantee (GTE)",
        "min_directors": 2,
        "max_directors": 20,
        "secretary_required": True,
        "min_share_capital": Decimal("0"),
        "residency": "At least 1 Nigerian director",
    },
    "unlimited_company": {
        "name": "Unlimited Company",
        "min_directors": 2,
        "max_directors": 50,
        "secretary_required": True,
        "min_share_capital": Decimal("0"),
        "residency": "No specific residency requirement",
    },
    "limited_liability_partnership": {
        "name": "Limited Liability Partnership (LLP)",
        "min_directors": 2,
        "max_directors": 50,
        "secretary_required": False,
        "min_share_capital": Decimal("0"),
        "residency": "At least 1 Nigerian partner",
    },
}


def check_company_registration_requirements(
    company_type: str,
) -> list[RegistrationRequirement]:
    """Check registration requirements for a company type under CAMA 2020.

    Args:
        company_type: One of 'private_limited', 'public_limited',
                     'company_limited_by_guarantee', 'unlimited_company',
                     'limited_liability_partnership'.

    Returns:
        List of RegistrationRequirement objects.
    """
    info = COMPANY_TYPES.get(company_type, COMPANY_TYPES["private_limited"])

    requirements = [
        RegistrationRequirement(
            requirement="Name Reservation",
            description="Reserve company name with CAC (valid for 60 days)",
            is_mandatory=True,
            applicable_to=[company_type],
        ),
        RegistrationRequirement(
            requirement="Memorandum of Association",
            description="Object clause defining company's business scope (CAMA 2020: no longer mandatory, may adopt general objects)",
            is_mandatory=False,
            applicable_to=[company_type],
        ),
        RegistrationRequirement(
            requirement="Articles of Association",
            description="Internal governance rules and regulations",
            is_mandatory=True,
            applicable_to=[company_type],
        ),
        RegistrationRequirement(
            requirement="Statement of Share Capital",
            description=f"Minimum share capital: NGN {info['min_share_capital']}",
            is_mandatory=True,
            applicable_to=[company_type],
        ),
        RegistrationRequirement(
            requirement="Directors Details",
            description=f"Minimum {info['min_directors']} directors, maximum {info['max_directors']}. {info['residency']}",
            is_mandatory=True,
            applicable_to=[company_type],
        ),
        RegistrationRequirement(
            requirement="Company Secretary",
            description=f"Company secretary {'required' if info['secretary_required'] else 'not required (CAMA 2020: optional for small private companies)'}",
            is_mandatory=info["secretary_required"],
            applicable_to=[company_type],
        ),
        RegistrationRequirement(
            requirement="Registered Address",
            description="Registered office address in Nigeria",
            is_mandatory=True,
            applicable_to=[company_type],
        ),
        RegistrationRequirement(
            requirement="CAC Forms",
            description="Complete CAC registration forms (CAC 1.1 for pre-registration, statutory forms)",
            is_mandatory=True,
            applicable_to=[company_type],
        ),
        RegistrationRequirement(
            requirement="Identification Documents",
            description="Valid identification for all directors and subscribers",
            is_mandatory=True,
            applicable_to=[company_type],
        ),
        RegistrationRequirement(
            requirement="Stamp Duty",
            description="Pay stamp duty on share capital and memorandum/articles",
            is_mandatory=True,
            applicable_to=[company_type],
        ),
    ]

    return requirements


def validate_annual_return_filing(
    company_data: dict[str, Any],
) -> AnnualReturnValidation:
    """Validate annual return filing requirements under CAMA 2020.

    Companies must file annual returns within 42 days of the AGM
    (or 18 months after incorporation for the first return).

    Args:
        company_data: Dict with 'name', 'company_type', 'has_filed',
                     'days_since_agm', 'filing_year', 'has_financials',
                     'has_director_changes'.

    Returns:
        AnnualReturnValidation with compliance status and required docs.
    """
    name = company_data.get("name", "")
    issues: list[str] = []
    required_docs: list[str] = []

    if not company_data.get("has_filed", False):
        days = company_data.get("days_since_agm", 0)
        if days > 42:
            issues.append(f"Annual return overdue by {days - 42} days (42-day limit after AGM)")

    required_docs = [
        "Audited financial statements",
        "Directors' report",
        "Auditor's report",
        "Annual return form (CAC 10)",
        "Statement of share capital and share allotment",
    ]

    if not company_data.get("has_financials", True):
        issues.append("Audited financial statements not available")
        required_docs.append("Audited financial statements - MISSING")

    if company_data.get("has_director_changes", False):
        required_docs.append("Notice of change in directors (CAC 7)")
        issues.append("Director changes require additional filing")

    return AnnualReturnValidation(
        company_name=name,
        is_valid=len(issues) == 0,
        issues=issues,
        required_documents=required_docs,
    )


def check_director_requirements(
    company_type: str,
) -> DirectorRequirement:
    """Check director requirements for a company type under CAMA 2020.

    Args:
        company_type: The company type classification string.

    Returns:
        DirectorRequirement with min/max directors and other rules.
    """
    info = COMPANY_TYPES.get(company_type, COMPANY_TYPES["private_limited"])

    return DirectorRequirement(
        company_type=company_type,
        minimum_directors=info["min_directors"],
        maximum_directors=info["max_directors"],
        secretary_required=info["secretary_required"],
        residency_requirement=info["residency"],
    )


from decimal import Decimal

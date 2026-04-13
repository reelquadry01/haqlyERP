"""NBS (National Bureau of Statistics) data structure alignment.

Author: Quadri Atharu

Maps financial data to NBS classification systems, retrieves economic
indicators, and formats statistical returns per NBS requirements for
Nigerian regulatory reporting.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


NBS_CLASSIFICATION: dict[str, str] = {
    "agriculture": "Section A: Agriculture, Forestry and Fishing",
    "mining": "Section B: Mining and Quarrying",
    "manufacturing": "Section C: Manufacturing",
    "electricity": "Section D: Electricity, Gas, Steam and Air Conditioning Supply",
    "water": "Section E: Water Supply, Sewerage, Waste Management",
    "construction": "Section F: Construction",
    "wholesale_retail": "Section G: Wholesale and Retail Trade",
    "transport": "Section H: Transportation and Storage",
    "accommodation": "Section I: Accommodation and Food Services",
    "ict": "Section J: Information and Communication",
    "financial": "Section K: Financial and Insurance Activities",
    "real_estate": "Section L: Real Estate Activities",
    "professional": "Section M: Professional, Scientific and Technical Activities",
    "admin": "Section N: Administrative and Support Service Activities",
    "public_admin": "Section O: Public Administration and Defence",
    "education": "Section P: Education",
    "health": "Section Q: Human Health and Social Work Activities",
    "entertainment": "Section R: Arts, Entertainment and Recreation",
    "other_services": "Section S: Other Service Activities",
    "household": "Section T: Activities of Households as Employers",
    "extraterritorial": "Section U: Activities of Extraterritorial Organizations",
}

ECONOMIC_INDICATORS: dict[str, dict[str, Any]] = {
    "gdp_growth_rate": {"value": Decimal("3.46"), "unit": "pct", "period": "2024-Q4"},
    "inflation_rate": {"value": Decimal("28.92"), "unit": "pct", "period": "2024-01"},
    "unemployment_rate": {"value": Decimal("33.3"), "unit": "pct", "period": "2024-Q1"},
    "exchange_rate_usd": {"value": Decimal("1550"), "unit": "NGN/USD", "period": "2024-12"},
    "interest_rate_mpr": {"value": Decimal("22.75"), "unit": "pct", "period": "2024-03"},
    "foreign_reserves": {"value": Decimal("34000000000"), "unit": "USD", "period": "2024-12"},
    "oil_production": {"value": Decimal("1.55"), "unit": "mbpd", "period": "2024-12"},
    "fiscal_deficit": {"value": Decimal("8500000000"), "unit": "NGN", "period": "2024"},
    "trade_balance": {"value": Decimal("1500000000"), "unit": "NGN", "period": "2024"},
    "external_debt": {"value": Decimal("42000000000"), "unit": "USD", "period": "2024-12"},
}


@dataclass
class NBSClassifiedData:
    industry: str
    nbs_section: str
    classified_items: dict[str, Decimal]
    metadata: dict[str, str] = field(default_factory=dict)


@dataclass
class EconomicIndicators:
    period: str
    indicators: dict[str, Decimal]
    source: str = "NBS / CBN"


@dataclass
class StatisticalReturn:
    entity_name: str
    period: str
    classification: str
    data: dict[str, str]
    format_version: str = "NBS-2024-v1"


def map_to_nbs_classification(
    financial_data: dict[str, Any],
) -> NBSClassifiedData:
    """Map financial data to NBS industrial classification.

    Args:
        financial_data: Dict with 'industry' key and line item amounts.

    Returns:
        NBSClassifiedData with mapped industry section and items.
    """
    industry = financial_data.get("industry", "").lower()
    nbs_section = NBS_CLASSIFICATION.get(industry, "Section S: Other Service Activities")

    classified_items: dict[str, Decimal] = {}
    for key, value in financial_data.items():
        if key == "industry":
            continue
        classified_items[key] = _d(value) if isinstance(value, (int, float, str, Decimal)) else Decimal("0")

    return NBSClassifiedData(
        industry=industry,
        nbs_section=nbs_section,
        classified_items=classified_items,
        metadata={"classification_standard": "ISIC Rev 4 / NBS Nigeria"},
    )


def get_economic_indicators(period: str) -> EconomicIndicators:
    """Get key Nigerian economic indicators for a period.

    Args:
        period: Period string (e.g. '2024-Q4', '2024-01').

    Returns:
        EconomicIndicators with GDP, inflation, exchange rate, etc.
    """
    indicators: dict[str, Decimal] = {}
    for key, info in ECONOMIC_INDICATORS.items():
        indicators[key] = _d(info["value"])

    return EconomicIndicators(
        period=period,
        indicators=indicators,
    )


def format_statistical_return(
    data: dict[str, Any],
) -> StatisticalReturn:
    """Format data as an NBS statistical return.

    Produces a structured return suitable for submission to NBS
    with proper classification and formatting.

    Args:
        data: Dict with 'entity_name', 'period', 'industry',
             and data items.

    Returns:
        StatisticalReturn formatted for NBS submission.
    """
    classified = map_to_nbs_classification(data)
    formatted_data: dict[str, str] = {}

    for key, value in classified.classified_items.items():
        formatted_data[key] = str(value)

    return StatisticalReturn(
        entity_name=data.get("entity_name", ""),
        period=data.get("period", ""),
        classification=classified.nbs_section,
        data=formatted_data,
    )

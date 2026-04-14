# Author: Quadri Atharu
"""Tax Engine module — Nigerian tax computation and compliance."""

from .vat import VatEngine
from .wht import WhtEngine
from .cit import CitEngine
from .education_tax import EducationTaxEngine
from .capital_gains_tax import CapitalGainsTaxEngine
from .stamp_duties import StampDutyEngine
from .tax_schedules import TaxScheduleGenerator
from .tax_returns import TaxReturnGenerator
from .tax_risk_flags import TaxRiskDetector

__all__ = [
    "VatEngine",
    "WhtEngine",
    "CitEngine",
    "EducationTaxEngine",
    "CapitalGainsTaxEngine",
    "StampDutyEngine",
    "TaxScheduleGenerator",
    "TaxReturnGenerator",
    "TaxRiskDetector",
]

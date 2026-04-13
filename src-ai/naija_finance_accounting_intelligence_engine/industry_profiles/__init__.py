"""Industry profile modules for sector-specific accounting intelligence.

Author: Quadri Atharu

Provides 15 industry-specific profile classes with tailored revenue models,
cost structures, tax implications, compliance requirements, inventory logic,
depreciation patterns, chart of accounts ranges, KPI computations, and
posting suggestion engines for Nigerian business sectors.
"""

from .oil_gas import OilGasProfile
from .manufacturing import ManufacturingProfile
from .banking import BankingProfile
from .insurance import InsuranceProfile
from .retail import RetailProfile
from .telecommunications import TelecommunicationsProfile
from .agriculture import AgricultureProfile
from .construction import ConstructionProfile
from .logistics import LogisticsProfile
from .healthcare import HealthcareProfile
from .education import EducationProfile
from .government import GovernmentProfile
from .ngo import NGOProfile
from .technology import TechnologyProfile
from .automotive import AutomotiveProfile

PROFILES = {
    "oil_gas": OilGasProfile,
    "manufacturing": ManufacturingProfile,
    "banking": BankingProfile,
    "insurance": InsuranceProfile,
    "retail": RetailProfile,
    "telecommunications": TelecommunicationsProfile,
    "agriculture": AgricultureProfile,
    "construction": ConstructionProfile,
    "logistics": LogisticsProfile,
    "healthcare": HealthcareProfile,
    "education": EducationProfile,
    "government": GovernmentProfile,
    "ngo": NGOProfile,
    "technology": TechnologyProfile,
    "automotive": AutomotiveProfile,
}

__all__ = list(PROFILES.keys()) + ["PROFILES", "OilGasProfile", "ManufacturingProfile", "BankingProfile", "InsuranceProfile", "RetailProfile", "TelecommunicationsProfile", "AgricultureProfile", "ConstructionProfile", "LogisticsProfile", "HealthcareProfile", "EducationProfile", "GovernmentProfile", "NGOProfile", "TechnologyProfile", "AutomotiveProfile"]

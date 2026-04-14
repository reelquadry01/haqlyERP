# Author: Quadri Atharu
"""Industry Analytics module — KPI engines for various Nigerian industries."""

from .manufacturing_kpi import ManufacturingKpiEngine
from .retail_kpi import RetailKpiEngine
from .banking_kpi import BankingKpiEngine
from .automotive_kpi import AutomotiveKpiEngine
from .custom_kpi import CustomKpiEngine

__all__ = ["ManufacturingKpiEngine", "RetailKpiEngine", "BankingKpiEngine", "AutomotiveKpiEngine", "CustomKpiEngine"]

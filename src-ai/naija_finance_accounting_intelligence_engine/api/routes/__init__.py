# Author: Quadri Atharu
"""API routes package for the HAQLY AI Finance Engine."""

from .accounting import router as accounting_router
from .tax import router as tax_router
from .reporting import router as reporting_router
from .treasury import router as treasury_router
from .analysis import router as analysis_router
from .audit import router as audit_router
from .erp import router as erp_router
from .agents import router as agents_router

__all__ = [
    "accounting_router",
    "tax_router",
    "reporting_router",
    "treasury_router",
    "analysis_router",
    "audit_router",
    "erp_router",
    "agents_router",
]

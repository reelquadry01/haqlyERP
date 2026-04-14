# Author: Quadri Atharu
"""HAQLY AI Finance Engine — FastAPI application entry point.

Nigerian Finance Intelligence Engine providing IFRS-compliant accounting,
Nigerian tax computation, financial reporting, treasury management,
financial analysis, audit intelligence, ERP integration, and agent-based automation.
"""

from __future__ import annotations

import logging
from contextlib import asynccontextmanager
from typing import AsyncGenerator, Dict, List

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from .routes import (
    accounting_router,
    tax_router,
    reporting_router,
    treasury_router,
    analysis_router,
    audit_router,
    erp_router,
    agents_router,
)

VERSION = "1.0.0"
MODULES: List[str] = [
    "accounting",
    "tax_engine",
    "reporting",
    "treasury",
    "financial_analysis",
    "audit_intelligence",
    "valuation",
    "erp_integration",
    "agents",
    "budgeting",
    "risk_management",
    "ifrs",
    "regulatory",
    "multi_company",
    "fx_accounting",
    "inflation_accounting",
    "industry_profiles",
    "working_capital",
    "internal_controls",
    "data_governance",
    "export",
]


@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncGenerator[None, None]:
    """Application lifespan handler — startup and shutdown logging."""
    logging.getLogger(__name__).info("AI Engine starting — HAQLY AI Finance Engine v%s", VERSION)
    yield
    logging.getLogger(__name__).info("AI Engine stopping — HAQLY AI Finance Engine v%s", VERSION)


app = FastAPI(
    title="HAQLY AI Finance Engine",
    version=VERSION,
    description="Nigerian Finance Intelligence Engine",
    lifespan=lifespan,
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "http://localhost:3001",
        "http://localhost:8100",
    ],
    allow_credentials=True,
    allow_methods=["GET", "POST", "PUT", "PATCH", "DELETE"],
    allow_headers=["Authorization", "Content-Type", "X-Company-Id"],
)

app.include_router(accounting_router)
app.include_router(tax_router)
app.include_router(reporting_router)
app.include_router(treasury_router)
app.include_router(analysis_router)
app.include_router(audit_router)
app.include_router(erp_router)
app.include_router(agents_router)


@app.get("/health")
async def health_check() -> Dict[str, object]:
    """Health check endpoint returning engine status and loaded modules."""
    return {
        "status": "healthy",
        "version": VERSION,
        "modules": MODULES,
    }

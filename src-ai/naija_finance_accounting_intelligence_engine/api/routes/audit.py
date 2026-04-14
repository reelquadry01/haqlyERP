# Author: Quadri Atharu
"""Audit routes — audit trail, sampling, and exception detection."""

from __future__ import annotations

from typing import Any, Dict, List, Optional

from fastapi import APIRouter, HTTPException, status

from ...audit_intelligence.audit_trail_generation import AuditTrailEngine
from ...audit_intelligence.sampling_logic import SamplingLogicEngine
from ...audit_intelligence.exception_detection import ExceptionDetectionEngine

router = APIRouter(prefix="/audit", tags=["Audit"])

_trail = AuditTrailEngine()
_sampling = SamplingLogicEngine()
_exceptions = ExceptionDetectionEngine()


@router.post("/trail")
async def generate_audit_trail(body: Dict[str, Any]) -> Dict[str, Any]:
    """Generate a full audit trail report for an entity and period."""
    entity = body.get("entity", "")
    period = body.get("period", "")
    if not entity or not period:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="entity and period are required")
    try:
        company_id = body.get("company_id", entity)
        period_start = body.get("period_start", period)
        period_end = body.get("period_end", period)
        result = _trail.generate_trail_report(
            company_id=company_id,
            period_start=period_start,
            period_end=period_end,
        )
        return {"status": "success", "audit_trail": result}
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))


@router.post("/sampling")
async def perform_sampling(body: Dict[str, Any]) -> Dict[str, Any]:
    """Perform statistical sampling for audit procedures."""
    population = body.get("population")
    sample_size = body.get("sample_size", 25)
    method = body.get("method", "random").lower()
    if not population:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="population is required")
    try:
        data = {
            "population": population,
            "sample_size": int(sample_size),
            "random_start": body.get("random_start", 0),
            "strata_key": body.get("strata_key", "type"),
            "confidence_level": body.get("confidence_level", 0.95),
            "tolerable_misstatement": body.get("tolerable_misstatement", 0),
            "value_key": body.get("value_key", "amount"),
        }
        if method == "systematic":
            result = _sampling.systematic_sample(data)
        elif method == "stratified":
            result = _sampling.stratified_sample(data)
        elif method == "monetary_unit":
            result = _sampling.monetary_unit_sample(data)
        else:
            result = _sampling.random_sample(data)
        return {"status": "success", "sampling": result}
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))


@router.post("/exceptions")
async def detect_exceptions(body: Dict[str, Any]) -> Dict[str, Any]:
    """Detect audit exceptions using Benford's law, duplicate detection, and outlier analysis."""
    data = body.get("data")
    rules = body.get("rules", [])
    if not data:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="data is required")
    try:
        results: Dict[str, Any] = {}

        amounts = [float(item.get("amount", 0)) for item in data if item.get("amount") is not None]
        if amounts:
            results["benford"] = _exceptions.benford_analysis(amounts)
            results["unusual_amounts"] = _exceptions.detect_unusual_amounts(
                amounts,
                threshold_std=float(rules.get("threshold_std", 2.5)) if isinstance(rules, dict) else 2.5,
            )

        key_fields = rules.get("duplicate_key_fields", ["reference", "amount"]) if isinstance(rules, dict) else ["reference", "amount"]
        results["duplicates"] = _exceptions.detect_duplicates(data, key_fields=key_fields)

        entries_with_dates = [d for d in data if d.get("entry_date") and d.get("period_end")]
        if entries_with_dates:
            results["late_entries"] = _exceptions.detect_late_entries(
                entries_with_dates,
                threshold_days=int(rules.get("late_entry_threshold_days", 5)) if isinstance(rules, dict) else 5,
            )

        return {"status": "success", "exceptions": results}
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))

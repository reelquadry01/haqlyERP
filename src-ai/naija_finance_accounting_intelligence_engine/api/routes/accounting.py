# Author: Quadri Atharu
"""Accounting routes — transaction processing, journal entries, trial balance, period closing."""

from __future__ import annotations

from typing import Any, Dict, Optional

from fastapi import APIRouter, HTTPException, status

from ...accounting.methods import AccountingMethods
from ...accounting.journal_engine import JournalEngine
from ...accounting.trial_balance import TrialBalanceEngine
from ...accounting.closing import ClosingEngine

router = APIRouter(prefix="/accounting", tags=["Accounting"])

_methods = AccountingMethods()
_journal_engine = JournalEngine()
_trial_balance = TrialBalanceEngine()
_closing = ClosingEngine()


@router.post("/process-transaction")
async def process_transaction(body: Dict[str, Any]) -> Dict[str, Any]:
    """Process a transaction under the accrual basis and return journal lines."""
    try:
        lines = _methods.process_accrual(body)
        return {"status": "success", "method": "accrual", "lines": lines}
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))


@router.post("/journal")
async def create_journal_entry(body: Dict[str, Any]) -> Dict[str, Any]:
    """Create a double-entry journal entry."""
    try:
        entry = _journal_engine.create_journal_entry(body)
        return {"status": "success", "entry": entry}
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))


@router.post("/trial-balance")
async def generate_trial_balance(body: Dict[str, Any]) -> Dict[str, Any]:
    """Generate a trial balance for a given company and period."""
    company_id = body.get("company_id", "")
    period_id = body.get("period_id", "")
    if not company_id or not period_id:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="company_id and period_id are required")
    try:
        tb = _trial_balance.generate_trial_balance(
            company_id=company_id,
            period_end=period_id,
            period_start=body.get("period_start"),
            include_zero_balances=body.get("include_zero_balances", False),
        )
        return {"status": "success", "trial_balance": tb}
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))


@router.post("/close-period")
async def close_period(body: Dict[str, Any]) -> Dict[str, Any]:
    """Close an accounting period (month-end or year-end)."""
    company_id = body.get("company_id", "")
    period_id = body.get("period_id", "")
    if not company_id or not period_id:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="company_id and period_id are required")
    closing_type = body.get("closing_type", "month_end")
    try:
        data = {
            "company_id": company_id,
            "period_end": period_id,
            "period_start": body.get("period_start"),
            "income_summary_account": body.get("income_summary_account", "3500"),
            "retained_earnings_account": body.get("retained_earnings_account", "3100"),
            "dividends_declared": body.get("dividends_declared", 0),
            "ledger_engine": body.get("ledger_engine"),
        }
        if closing_type == "year_end":
            data.update({
                "fiscal_year_end": period_id,
                "tax_provision": body.get("tax_provision", 0),
                "bonus_provision": body.get("bonus_provision", 0),
                "transfer_to_reserve": body.get("transfer_to_reserve", 0),
            })
            result = _closing.close_year(data)
        else:
            result = _closing.close_month(data)
        return {"status": "success", "closing": result}
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))

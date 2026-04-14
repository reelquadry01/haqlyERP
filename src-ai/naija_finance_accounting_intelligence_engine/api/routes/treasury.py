# Author: Quadri Atharu
"""Treasury routes — cash position, bank reconciliation, loan amortization."""

from __future__ import annotations

from typing import Any, Dict, Optional

from fastapi import APIRouter, HTTPException, status

from ...treasury.cash_position import CashPositionEngine
from ...treasury.bank_reconciliation import BankReconciliationEngine
from ...treasury.loan_management import LoanManagementEngine

router = APIRouter(prefix="/treasury", tags=["Treasury"])

_cash_position = CashPositionEngine()
_bank_reconciliation = BankReconciliationEngine()
_loan_management = LoanManagementEngine()


@router.get("/cash-position")
async def get_cash_position(company_id: str) -> Dict[str, Any]:
    """Monitor current cash position for a company."""
    try:
        result = _cash_position.compute_cash_position({"company_id": company_id})
        return {"status": "success", "cash_position": result}
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))


@router.post("/bank-reconciliation")
async def perform_bank_reconciliation(body: Dict[str, Any]) -> Dict[str, Any]:
    """Perform bank reconciliation between book records and bank statement."""
    book_balance = body.get("book_balance")
    bank_balance = body.get("bank_balance")
    if book_balance is None or bank_balance is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="book_balance and bank_balance are required")
    try:
        data = {
            "company_id": body.get("company_id", ""),
            "book_balance": float(book_balance),
            "bank_balance": float(bank_balance),
            "bank_name": body.get("bank_name", ""),
            "account_number": body.get("account_number", ""),
            "period_end": body.get("period_end"),
            "currency": body.get("currency", "NGN"),
            "outstanding_deposits": body.get("outstanding_deposits", []),
            "outstanding_cheques": body.get("outstanding_cheques", []),
            "bank_charges": body.get("bank_charges", 0),
            "bank_interest": body.get("bank_interest", 0),
            "errors": body.get("errors", []),
        }
        result = _bank_reconciliation.reconcile(data)
        return {"status": "success", "reconciliation": result}
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))


@router.get("/loan-schedule")
async def get_loan_schedule(loan_id: str) -> Dict[str, Any]:
    """Generate an amortization schedule for a loan."""
    try:
        data = {
            "loan_id": loan_id,
            "principal": float(0),
            "annual_rate": 0.20,
            "term_months": 12,
        }
        result = _loan_management.generate_amortization_schedule(data)
        return {"status": "success", "loan_schedule": result}
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))

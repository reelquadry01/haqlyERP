# Author: Quadri Atharu
"""Loan management and amortization schedule engine."""

from __future__ import annotations

from datetime import datetime, timedelta
from typing import Any, Dict, List, Optional

from ..core.exceptions import AccountingError
from ..core.logging import get_logger
from decimal import Decimal, ROUND_HALF_UP


def _money_round(value) -> Decimal:
    if isinstance(value, Decimal):
        return value.quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)
    return Decimal(str(value)).quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)


logger = get_logger(__name__)


class LoanManagementEngine:
    """Loan management engine with amortization schedule generation."""

    def generate_amortization_schedule(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate a loan amortization schedule."""
        principal = Decimal(str(data.get("principal", 0)))
        annual_rate = Decimal(str(data.get("annual_rate", 0.20)))
        term_months = int(data.get("term_months", 12))
        repayment_type = data.get("repayment_type", "reducing_balance").lower()
        start_date_str = data.get("start_date", datetime.now().isoformat())
        payment_frequency = data.get("payment_frequency", "monthly").lower()

        if principal <= 0:
            raise AccountingError("Loan principal must be positive")
        if term_months <= 0:
            raise AccountingError("Loan term must be positive")
        if annual_rate < 0:
            raise AccountingError("Interest rate cannot be negative")

        try:
            start_date = datetime.fromisoformat(start_date_str)
        except (ValueError, TypeError):
            start_date = datetime.now()

        frequency_months = {"monthly": 1, "quarterly": 3, "annually": 12}
        freq_months = frequency_months.get(payment_frequency, 1)
        num_payments = term_months // freq_months
        if num_payments <= 0:
            num_payments = 1
        periodic_rate = annual_rate * freq_months / 12

        if repayment_type == "reducing_balance":
            schedule, emi = self._reducing_balance_schedule(principal, periodic_rate, num_payments, start_date, freq_months)
        elif repayment_type == "flat":
            schedule, emi = self._flat_rate_schedule(principal, periodic_rate, num_payments, start_date, freq_months)
        elif repayment_type == "interest_only":
            schedule, emi = self._interest_only_schedule(principal, periodic_rate, num_payments, start_date, freq_months)
        else:
            raise AccountingError(f"Unknown repayment type: {repayment_type}")

        total_payment = round(sum(p.get("payment", 0) for p in schedule), 2)
        total_interest = round(sum(p.get("interest", 0) for p in schedule), 2)
        total_principal_paid = round(sum(p.get("principal_payment", 0) for p in schedule), 2)

        result: Dict[str, Any] = {
            "loan_id": data.get("loan_id", ""),
            "loan_name": data.get("loan_name", ""),
            "principal": _money_round(principal),
            "annual_rate": annual_rate,
            "periodic_rate": periodic_rate,
            "term_months": term_months,
            "repayment_type": repayment_type,
            "payment_frequency": payment_frequency,
            "emi": _money_round(emi),
            "num_payments": num_payments,
            "total_payment": total_payment,
            "total_interest": total_interest,
            "total_principal_repaid": total_principal_paid,
            "schedule": schedule,
            "currency": data.get("currency", "NGN"),
            "start_date": start_date.isoformat(),
            "generated_at": datetime.now().isoformat(),
        }

        logger.info("amortization_schedule_generated", principal=principal, term=term_months, type=repayment_type)
        return result

    def compute_outstanding_balance(self, schedule: List[Dict[str, Any]], periods_paid: int) -> Dict[str, Any]:
        """Compute outstanding loan balance after a number of payments."""
        if periods_paid >= len(schedule):
            return {"outstanding_balance": 0.0, "loan_fully_repaid": True, "periods_paid": periods_paid}

        total_paid = round(sum(schedule[i].get("payment", 0) for i in range(min(periods_paid, len(schedule)))), 2)
        outstanding = schedule[min(periods_paid, len(schedule) - 1)].get("closing_balance", 0)

        return {
            "periods_paid": periods_paid,
            "total_paid": total_paid,
            "outstanding_balance": _money_round(outstanding),
            "loan_fully_repaid": periods_paid >= len(schedule),
            "next_payment_due": schedule[periods_paid].get("payment_date") if periods_paid < len(schedule) else None,
        }

    def early_settlement_analysis(self, schedule: List[Dict[str, Any]], current_period: int, early_settlement_fee_pct: float = 0.0) -> Dict[str, Any]:
        """Analyze early settlement of a loan."""
        if current_period >= len(schedule):
            return {"message": "Loan already fully repaid"}

        outstanding = schedule[current_period].get("closing_balance", 0)
        remaining_interest = round(sum(schedule[i].get("interest", 0) for i in range(current_period, len(schedule))), 2)
        remaining_payments = round(sum(schedule[i].get("payment", 0) for i in range(current_period, len(schedule))), 2)
        settlement_fee = _money_round(outstanding * early_settlement_fee_pct)
        total_early_cost = _money_round(outstanding + settlement_fee)
        interest_saving = _money_round(remaining_interest - settlement_fee)

        return {
            "outstanding_principal": _money_round(outstanding),
            "remaining_scheduled_payments": remaining_payments,
            "remaining_interest": remaining_interest,
            "early_settlement_fee_pct": early_settlement_fee_pct,
            "early_settlement_fee": settlement_fee,
            "total_early_settlement_cost": total_early_cost,
            "interest_saving": interest_saving,
            "beneficial": interest_saving > 0,
            "recommendation": "Early settlement is beneficial" if interest_saving > 0 else "Consider keeping the loan — early settlement fee may exceed interest savings",
        }

    def _reducing_balance_schedule(self, principal: float, periodic_rate: float, num_payments: int, start_date: datetime, freq_months: int) -> tuple:
        """Generate reducing balance amortization schedule."""
        if periodic_rate == 0:
            emi = principal / num_payments
        else:
            emi = principal * periodic_rate * (1 + periodic_rate) ** num_payments / ((1 + periodic_rate) ** num_payments - 1)

        schedule: List[Dict[str, Any]] = []
        balance = principal
        for p in range(1, num_payments + 1):
            interest = _money_round(balance * periodic_rate)
            payment = round(min(emi, balance + interest), 2)
            principal_payment = _money_round(payment - interest)
            balance = round(max(balance - principal_payment, 0), 2)
            payment_date = start_date + timedelta(days=30 * freq_months * p)

            schedule.append({
                "period": p,
                "payment_date": payment_date.isoformat(),
                "opening_balance": _money_round(balance + principal_payment),
                "payment": payment,
                "interest": interest,
                "principal_payment": principal_payment,
                "closing_balance": balance,
            })

        return schedule, emi

    def _flat_rate_schedule(self, principal: float, periodic_rate: float, num_payments: int, start_date: datetime, freq_months: int) -> tuple:
        """Generate flat rate amortization schedule."""
        total_interest = _money_round(principal * periodic_rate * num_payments)
        total_payment = _money_round(principal + total_interest)
        emi = _money_round(total_payment / num_payments)
        interest_per_period = _money_round(total_interest / num_payments)
        principal_per_period = _money_round(principal / num_payments)

        schedule: List[Dict[str, Any]] = []
        balance = principal
        for p in range(1, num_payments + 1):
            payment_date = start_date + timedelta(days=30 * freq_months * p)
            balance = round(max(balance - principal_per_period, 0), 2)

            schedule.append({
                "period": p,
                "payment_date": payment_date.isoformat(),
                "opening_balance": _money_round(balance + principal_per_period),
                "payment": emi,
                "interest": interest_per_period,
                "principal_payment": principal_per_period,
                "closing_balance": balance,
            })

        return schedule, emi

    def _interest_only_schedule(self, principal: float, periodic_rate: float, num_payments: int, start_date: datetime, freq_months: int) -> tuple:
        """Generate interest-only amortization schedule with balloon payment."""
        interest_per_period = _money_round(principal * periodic_rate)
        schedule: List[Dict[str, Any]] = []

        for p in range(1, num_payments + 1):
            payment_date = start_date + timedelta(days=30 * freq_months * p)
            is_last = p == num_payments
            payment = _money_round(interest_per_period + (principal if is_last else 0))

            schedule.append({
                "period": p,
                "payment_date": payment_date.isoformat(),
                "opening_balance": principal,
                "payment": payment,
                "interest": interest_per_period,
                "principal_payment": principal if is_last else 0,
                "closing_balance": 0 if is_last else principal,
            })

        emi = interest_per_period
        return schedule, emi

    def health_check(self) -> bool:
        return True

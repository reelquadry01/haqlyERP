# Author: Quadri Atharu
"""Cash position monitoring and forecasting."""

from __future__ import annotations

from datetime import datetime, timedelta
from typing import Any, Dict, List, Optional

from ..core.logging import get_logger
from decimal import Decimal, ROUND_HALF_UP


def _money_round(value) -> Decimal:
    if isinstance(value, Decimal):
        return value.quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)
    return Decimal(str(value)).quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)


logger = get_logger(__name__)


class CashPositionEngine:
    """Cash position monitoring and forecasting engine."""

    def compute_cash_position(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute current cash position from bank balances and cash on hand."""
        bank_balances: Dict[str, float] = data.get("bank_balances", {})
        petty_cash = Decimal(str(data.get("petty_cash", 0)))
        undeposited_funds = Decimal(str(data.get("undeposited_funds", 0)))
        restricted_funds = Decimal(str(data.get("restricted_funds", 0)))
        outstanding_cheques = Decimal(str(data.get("outstanding_cheques", 0)))
        outstanding_deposits = Decimal(str(data.get("outstanding_deposits", 0)))

        total_bank = round(sum(v for v in bank_balances.values() if isinstance(v, (int, float))), 2)
        total_available = _money_round(total_bank + petty_cash + undeposited_funds - restricted_funds)
        book_balance = _money_round(total_available + outstanding_deposits - outstanding_cheques)

        return {
            "company_id": data.get("company_id", ""),
            "as_of": data.get("as_of", datetime.now().isoformat()),
            "currency": data.get("currency", "NGN"),
            "bank_accounts": [{"bank": k, "balance": _money_round(v)} for k, v in bank_balances.items() if isinstance(v, (int, float))],
            "total_bank_balances": total_bank,
            "petty_cash": _money_round(petty_cash),
            "undeposited_funds": _money_round(undeposited_funds),
            "restricted_funds": _money_round(restricted_funds),
            "outstanding_cheques": _money_round(outstanding_cheques),
            "outstanding_deposits": _money_round(outstanding_deposits),
            "total_available_cash": total_available,
            "book_balance": book_balance,
            "computed_at": datetime.now().isoformat(),
        }

    def forecast_cash_position(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Forecast cash position for a number of days ahead."""
        current_balance = Decimal(str(data.get("current_balance", 0)))
        daily_inflow_avg = Decimal(str(data.get("daily_inflow_avg", 0)))
        daily_outflow_avg = Decimal(str(data.get("daily_outflow_avg", 0)))
        forecast_days = int(data.get("forecast_days", 30))
        scheduled_inflows: List[Dict[str, Any]] = data.get("scheduled_inflows", [])
        scheduled_outflows: List[Dict[str, Any]] = data.get("scheduled_outflows", [])
        minimum_balance = Decimal(str(data.get("minimum_balance", 0)))

        daily_net = _money_round(daily_inflow_avg - daily_outflow_avg)
        projected_base = _money_round(current_balance + daily_net * forecast_days)

        scheduled_in = round(sum(float(s.get("amount", 0)) for s in scheduled_inflows), 2)
        scheduled_out = round(sum(float(s.get("amount", 0)) for s in scheduled_outflows), 2)

        projected_balance = _money_round(projected_base + scheduled_in - scheduled_out)
        deficit = projected_balance < minimum_balance

        daily_positions: List[Dict[str, Any]] = []
        running = current_balance
        for day in range(1, forecast_days + 1):
            day_inflow = sum(float(s.get("amount", 0)) for s in scheduled_inflows if int(s.get("days_from_now", 0)) == day)
            day_outflow = sum(float(s.get("amount", 0)) for s in scheduled_outflows if int(s.get("days_from_now", 0)) == day)
            running = _money_round(running + daily_net + day_inflow - day_outflow)
            daily_positions.append({"day": day, "projected_balance": running, "daily_inflow": _money_round(daily_inflow_avg + day_inflow), "daily_outflow": _money_round(daily_outflow_avg + day_outflow)})

        shortfall_day = None
        for dp in daily_positions:
            if dp["projected_balance"] < minimum_balance:
                shortfall_day = dp["day"]
                break

        result: Dict[str, Any] = {
            "current_balance": _money_round(current_balance),
            "daily_net_cash_flow": daily_net,
            "forecast_days": forecast_days,
            "scheduled_inflows_total": scheduled_in,
            "scheduled_outflows_total": scheduled_out,
            "projected_balance": projected_balance,
            "minimum_balance_threshold": _money_round(minimum_balance),
            "deficit_projected": deficit,
            "shortfall_day": shortfall_day,
            "daily_positions": daily_positions,
            "recommendation": "Arrange short-term financing" if deficit else "Cash position is healthy",
        }

        if deficit:
            shortfall_amount = _money_round(minimum_balance - projected_balance)
            result["shortfall_amount"] = shortfall_amount
            result["financing_needed"] = max(shortfall_amount, 0)

        logger.info("cash_forecast_generated", projected=projected_balance, deficit=deficit, days=forecast_days)
        return result

    def compute_cash_utilization(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Analyze cash utilization efficiency."""
        total_available = Decimal(str(data.get("total_available_cash", 0)))
        operating_expenses_monthly = Decimal(str(data.get("operating_expenses_monthly", 0)))
        cash_in_transit = Decimal(str(data.get("cash_in_transit", 0)))
        idle_cash = Decimal(str(data.get("idle_cash", 0)))

        months_cover = _money_round(total_available / operating_expenses_monthly) if operating_expenses_monthly > 0 else 0
        utilization_rate = round((total_available - idle_cash) / total_available, 4) if total_available > 0 else 0

        return {
            "total_available_cash": _money_round(total_available),
            "monthly_operating_expenses": _money_round(operating_expenses_monthly),
            "months_of_expenses_covered": months_cover,
            "cash_in_transit": _money_round(cash_in_transit),
            "idle_cash": _money_round(idle_cash),
            "cash_utilization_rate": utilization_rate,
            "utilization_pct": f"{utilization_rate * 100:.1f}%",
            "recommendation": "Consider investing idle cash in short-term instruments" if idle_cash > operating_expenses_monthly else "Cash utilization is efficient",
        }

    def health_check(self) -> bool:
        return True

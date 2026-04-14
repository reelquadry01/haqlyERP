# Author: Quadri Atharu
"""Cash position monitoring and forecasting."""

from __future__ import annotations

from datetime import datetime, timedelta
from typing import Any, Dict, List, Optional

from ..core.logging import get_logger

logger = get_logger(__name__)


class CashPositionEngine:
    """Cash position monitoring and forecasting engine."""

    def compute_cash_position(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute current cash position from bank balances and cash on hand."""
        bank_balances: Dict[str, float] = data.get("bank_balances", {})
        petty_cash = float(data.get("petty_cash", 0))
        undeposited_funds = float(data.get("undeposited_funds", 0))
        restricted_funds = float(data.get("restricted_funds", 0))
        outstanding_cheques = float(data.get("outstanding_cheques", 0))
        outstanding_deposits = float(data.get("outstanding_deposits", 0))

        total_bank = round(sum(v for v in bank_balances.values() if isinstance(v, (int, float))), 2)
        total_available = round(total_bank + petty_cash + undeposited_funds - restricted_funds, 2)
        book_balance = round(total_available + outstanding_deposits - outstanding_cheques, 2)

        return {
            "company_id": data.get("company_id", ""),
            "as_of": data.get("as_of", datetime.now().isoformat()),
            "currency": data.get("currency", "NGN"),
            "bank_accounts": [{"bank": k, "balance": round(v, 2)} for k, v in bank_balances.items() if isinstance(v, (int, float))],
            "total_bank_balances": total_bank,
            "petty_cash": round(petty_cash, 2),
            "undeposited_funds": round(undeposited_funds, 2),
            "restricted_funds": round(restricted_funds, 2),
            "outstanding_cheques": round(outstanding_cheques, 2),
            "outstanding_deposits": round(outstanding_deposits, 2),
            "total_available_cash": total_available,
            "book_balance": book_balance,
            "computed_at": datetime.now().isoformat(),
        }

    def forecast_cash_position(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Forecast cash position for a number of days ahead."""
        current_balance = float(data.get("current_balance", 0))
        daily_inflow_avg = float(data.get("daily_inflow_avg", 0))
        daily_outflow_avg = float(data.get("daily_outflow_avg", 0))
        forecast_days = int(data.get("forecast_days", 30))
        scheduled_inflows: List[Dict[str, Any]] = data.get("scheduled_inflows", [])
        scheduled_outflows: List[Dict[str, Any]] = data.get("scheduled_outflows", [])
        minimum_balance = float(data.get("minimum_balance", 0))

        daily_net = round(daily_inflow_avg - daily_outflow_avg, 2)
        projected_base = round(current_balance + daily_net * forecast_days, 2)

        scheduled_in = round(sum(float(s.get("amount", 0)) for s in scheduled_inflows), 2)
        scheduled_out = round(sum(float(s.get("amount", 0)) for s in scheduled_outflows), 2)

        projected_balance = round(projected_base + scheduled_in - scheduled_out, 2)
        deficit = projected_balance < minimum_balance

        daily_positions: List[Dict[str, Any]] = []
        running = current_balance
        for day in range(1, forecast_days + 1):
            day_inflow = sum(float(s.get("amount", 0)) for s in scheduled_inflows if int(s.get("days_from_now", 0)) == day)
            day_outflow = sum(float(s.get("amount", 0)) for s in scheduled_outflows if int(s.get("days_from_now", 0)) == day)
            running = round(running + daily_net + day_inflow - day_outflow, 2)
            daily_positions.append({"day": day, "projected_balance": running, "daily_inflow": round(daily_inflow_avg + day_inflow, 2), "daily_outflow": round(daily_outflow_avg + day_outflow, 2)})

        shortfall_day = None
        for dp in daily_positions:
            if dp["projected_balance"] < minimum_balance:
                shortfall_day = dp["day"]
                break

        result: Dict[str, Any] = {
            "current_balance": round(current_balance, 2),
            "daily_net_cash_flow": daily_net,
            "forecast_days": forecast_days,
            "scheduled_inflows_total": scheduled_in,
            "scheduled_outflows_total": scheduled_out,
            "projected_balance": projected_balance,
            "minimum_balance_threshold": round(minimum_balance, 2),
            "deficit_projected": deficit,
            "shortfall_day": shortfall_day,
            "daily_positions": daily_positions,
            "recommendation": "Arrange short-term financing" if deficit else "Cash position is healthy",
        }

        if deficit:
            shortfall_amount = round(minimum_balance - projected_balance, 2)
            result["shortfall_amount"] = shortfall_amount
            result["financing_needed"] = max(shortfall_amount, 0)

        logger.info("cash_forecast_generated", projected=projected_balance, deficit=deficit, days=forecast_days)
        return result

    def compute_cash_utilization(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Analyze cash utilization efficiency."""
        total_available = float(data.get("total_available_cash", 0))
        operating_expenses_monthly = float(data.get("operating_expenses_monthly", 0))
        cash_in_transit = float(data.get("cash_in_transit", 0))
        idle_cash = float(data.get("idle_cash", 0))

        months_cover = round(total_available / operating_expenses_monthly, 2) if operating_expenses_monthly > 0 else 0
        utilization_rate = round((total_available - idle_cash) / total_available, 4) if total_available > 0 else 0

        return {
            "total_available_cash": round(total_available, 2),
            "monthly_operating_expenses": round(operating_expenses_monthly, 2),
            "months_of_expenses_covered": months_cover,
            "cash_in_transit": round(cash_in_transit, 2),
            "idle_cash": round(idle_cash, 2),
            "cash_utilization_rate": utilization_rate,
            "utilization_pct": f"{utilization_rate * 100:.1f}%",
            "recommendation": "Consider investing idle cash in short-term instruments" if idle_cash > operating_expenses_monthly else "Cash utilization is efficient",
        }

    def health_check(self) -> bool:
        return True

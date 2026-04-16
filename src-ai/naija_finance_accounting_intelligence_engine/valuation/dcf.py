# Author: Quadri Atharu
"""Discounted Cash Flow (DCF) valuation engine with terminal value."""

from __future__ import annotations

from datetime import datetime
from decimal import Decimal, ROUND_HALF_UP
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)


def _money_round(value) -> Decimal:
    if isinstance(value, Decimal):
        return value.quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)
    return Decimal(str(value)).quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)


class DcfEngine:
    """DCF valuation engine with terminal value computation."""

    def compute_dcf(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute DCF valuation with terminal value."""
        cash_flows = [Decimal(str(cf)) for cf in data.get("cash_flows", [])]
        discount_rate = float(data.get("discount_rate", 0.10))
        terminal_growth_rate = float(data.get("terminal_growth_rate", 0.03))
        terminal_value_method = data.get("terminal_value_method", "gordon_growth").lower()
        net_debt = Decimal(str(data.get("net_debt", 0)))
        shares_outstanding = Decimal(str(data.get("shares_outstanding", 1)))

        pv_flows = Decimal('0')
        pv_details: List[Dict[str, Any]] = []

        for i, cf in enumerate(cash_flows):
            discount_factor = Decimal(str((1 + discount_rate) ** (i + 1)))
            pv = _money_round(cf / discount_factor)
            pv_flows += pv
            pv_details.append({"year": i + 1, "cash_flow": float(cf), "present_value": pv})

        last_cf = cash_flows[-1] if cash_flows else Decimal('0')

        if terminal_value_method == "gordon_growth":
            if discount_rate > terminal_growth_rate:
                terminal_value = _money_round(last_cf * Decimal(str(1 + terminal_growth_rate)) / Decimal(str(discount_rate - terminal_growth_rate)))
            else:
                terminal_value = Decimal('0')
        elif terminal_value_method == "exit_multiple":
            exit_multiple = Decimal(str(data.get("exit_multiple", 8)))
            terminal_value = _money_round(last_cf * exit_multiple)
        else:
            terminal_value = Decimal('0')

        n = len(cash_flows)
        pv_terminal = _money_round(terminal_value / Decimal(str((1 + discount_rate) ** n))) if discount_rate > 0 else terminal_value
        enterprise_value = _money_round(pv_flows + pv_terminal)
        equity_value = _money_round(enterprise_value - net_debt)
        per_share = _money_round(equity_value / shares_outstanding) if shares_outstanding > 0 else Decimal('0')

        return {
            "discount_rate": discount_rate,
            "terminal_growth_rate": terminal_growth_rate,
            "terminal_value_method": terminal_value_method,
            "explicit_period_years": n,
            "pv_of_explicit_cash_flows": _money_round(pv_flows),
            "terminal_value": terminal_value,
            "pv_of_terminal_value": pv_terminal,
            "enterprise_value": enterprise_value,
            "net_debt": _money_round(net_debt),
            "equity_value": equity_value,
            "shares_outstanding": float(shares_outstanding),
            "value_per_share": per_share,
            "terminal_value_pct_of_ev": round(float(pv_terminal) / float(enterprise_value), 4) if enterprise_value > 0 else 0,
            "pv_details": pv_details,
            "computed_at": datetime.now().isoformat(),
        }

    def compute_dcf_sensitivity(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute DCF sensitivity across discount rates and growth rates."""
        cash_flows_float: List[float] = [float(cf) for cf in data.get("cash_flows", [])]
        base_wacc = float(data.get("discount_rate", 0.10))
        base_growth = float(data.get("terminal_growth_rate", 0.03))
        net_debt = Decimal(str(data.get("net_debt", 0)))
        shares = Decimal(str(data.get("shares_outstanding", 1)))

        wacc_range = [round(base_wacc + delta * 0.01, 4) for delta in range(-3, 4)]
        growth_range = [round(base_growth + delta * 0.005, 4) for delta in range(-3, 4)]

        matrix: List[List[Any]] = []
        for wacc in wacc_range:
            row: List[Any] = []
            for growth in growth_range:
                if wacc <= growth:
                    row.append(None)
                    continue
                result = self.compute_dcf({
                    "cash_flows": cash_flows_float,
                    "discount_rate": wacc,
                    "terminal_growth_rate": growth,
                    "net_debt": net_debt,
                    "shares_outstanding": shares,
                })
                row.append(result.get("value_per_share", 0))
            matrix.append(row)

        return {
            "wacc_range": wacc_range,
            "growth_range": growth_range,
            "sensitivity_matrix": matrix,
            "note": "Values represent equity value per share at each WACC/growth combination",
        }

    def health_check(self) -> bool:
        return True

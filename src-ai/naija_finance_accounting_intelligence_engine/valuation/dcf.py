# Author: Quadri Atharu
"""Discounted Cash Flow (DCF) valuation engine with terminal value."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)


class DcfEngine:
    """DCF valuation engine with terminal value computation."""

    def compute_dcf(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute DCF valuation with terminal value."""
        cash_flows: List[float] = [float(cf) for cf in data.get("cash_flows", [])]
        discount_rate = float(data.get("discount_rate", 0.10))
        terminal_growth_rate = float(data.get("terminal_growth_rate", 0.03))
        terminal_value_method = data.get("terminal_value_method", "gordon_growth").lower()
        net_debt = float(data.get("net_debt", 0))
        shares_outstanding = float(data.get("shares_outstanding", 1))

        pv_flows = 0.0
        pv_details: List[Dict[str, Any]] = []

        for i, cf in enumerate(cash_flows):
            pv = round(cf / (1 + discount_rate) ** (i + 1), 2)
            pv_flows += pv
            pv_details.append({"year": i + 1, "cash_flow": cf, "present_value": pv})

        last_cf = cash_flows[-1] if cash_flows else 0

        if terminal_value_method == "gordon_growth":
            terminal_value = round(last_cf * (1 + terminal_growth_rate) / (discount_rate - terminal_growth_rate), 2) if discount_rate > terminal_growth_rate else 0
        elif terminal_value_method == "exit_multiple":
            exit_multiple = float(data.get("exit_multiple", 8))
            terminal_value = round(last_cf * exit_multiple, 2)
        else:
            terminal_value = 0

        n = len(cash_flows)
        pv_terminal = round(terminal_value / (1 + discount_rate) ** n, 2) if discount_rate > 0 else terminal_value
        enterprise_value = round(pv_flows + pv_terminal, 2)
        equity_value = round(enterprise_value - net_debt, 2)
        per_share = round(equity_value / shares_outstanding, 2) if shares_outstanding > 0 else 0

        return {
            "discount_rate": discount_rate,
            "terminal_growth_rate": terminal_growth_rate,
            "terminal_value_method": terminal_value_method,
            "explicit_period_years": n,
            "pv_of_explicit_cash_flows": round(pv_flows, 2),
            "terminal_value": terminal_value,
            "pv_of_terminal_value": pv_terminal,
            "enterprise_value": enterprise_value,
            "net_debt": round(net_debt, 2),
            "equity_value": equity_value,
            "shares_outstanding": shares_outstanding,
            "value_per_share": per_share,
            "terminal_value_pct_of_ev": round(pv_terminal / enterprise_value, 4) if enterprise_value > 0 else 0,
            "pv_details": pv_details,
            "computed_at": datetime.now().isoformat(),
        }

    def compute_dcf_sensitivity(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute DCF sensitivity across discount rates and growth rates."""
        cash_flows: List[float] = [float(cf) for cf in data.get("cash_flows", [])]
        base_wacc = float(data.get("discount_rate", 0.10))
        base_growth = float(data.get("terminal_growth_rate", 0.03))
        net_debt = float(data.get("net_debt", 0))
        shares = float(data.get("shares_outstanding", 1))

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
                    "cash_flows": cash_flows,
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

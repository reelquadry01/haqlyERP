# Author: Quadri Atharu
"""Internal Rate of Return (IRR) computation engine using numpy."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List, Optional

from ..core.exceptions import AnalysisError
from ..core.logging import get_logger

logger = get_logger(__name__)

try:
    import numpy as np
    HAS_NUMPY = True
except ImportError:
    HAS_NUMPY = False


class IrrEngine:
    """Internal Rate of Return computation engine."""

    def compute_irr(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute IRR given cash flows including initial investment."""
        cash_flows: List[float] = [float(cf) for cf in data.get("cash_flows", [])]

        if len(cash_flows) < 2:
            return {"irr": None, "message": "At least 2 cash flows required"}

        if HAS_NUMPY:
            irr = self._numpy_irr(cash_flows)
        else:
            irr = self._manual_irr(cash_flows)

        if irr is None:
            return {"irr": None, "message": "IRR could not be computed — cash flows may not cross zero"}

        npv_at_irr = self._compute_npv(cash_flows, irr)

        return {
            "irr": round(irr, 6),
            "irr_pct": f"{irr * 100:.2f}%",
            "npv_at_irr": round(npv_at_irr, 4),
            "cash_flows": cash_flows,
            "recommendation": f"Accept if IRR > cost of capital ({data.get('cost_of_capital', 'not specified')})",
            "computed_at": datetime.now().isoformat(),
        }

    def compute_mirr(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute Modified Internal Rate of Return."""
        cash_flows: List[float] = [float(cf) for cf in data.get("cash_flows", [])]
        finance_rate = float(data.get("finance_rate", 0.10))
        reinvest_rate = float(data.get("reinvest_rate", 0.10))

        if len(cash_flows) < 2:
            return {"mirr": None, "message": "At least 2 cash flows required"}

        n = len(cash_flows) - 1

        positive_flows = [cf for cf in cash_flows[1:] if cf > 0]
        negative_flows = [cf for cf in cash_flows[1:] if cf < 0]

        fv_positive = 0.0
        for i, cf in enumerate(cash_flows[1:]):
            if cf > 0:
                fv_positive += cf * (1 + reinvest_rate) ** (n - i - 1)

        pv_negative = abs(cash_flows[0])
        for i, cf in enumerate(cash_flows[1:]):
            if cf < 0:
                pv_negative += abs(cf) / (1 + finance_rate) ** (i + 1)

        if pv_negative == 0:
            return {"mirr": None, "message": "No negative cash flows"}

        mirr = (fv_positive / pv_negative) ** (1 / n) - 1 if n > 0 else 0

        return {
            "mirr": round(mirr, 6),
            "mirr_pct": f"{mirr * 100:.2f}%",
            "finance_rate": finance_rate,
            "reinvest_rate": reinvest_rate,
            "computed_at": datetime.now().isoformat(),
        }

    @staticmethod
    def _numpy_irr(cash_flows: List[float]) -> Optional[float]:
        """Compute IRR using numpy."""
        try:
            return float(np.irr(cash_flows)) if hasattr(np, 'irr') else float(np.financial.irr(cash_flows))
        except (AttributeError, ValueError):
            try:
                return float(np.irr(cash_flows))
            except Exception:
                return IrrEngine._manual_irr(cash_flows)

    @staticmethod
    def _manual_irr(cash_flows: List[float], max_iterations: int = 1000, tolerance: float = 1e-8) -> Optional[float]:
        """Compute IRR using Newton-Raphson method."""
        rate = 0.1
        for _ in range(max_iterations):
            npv = sum(cf / (1 + rate) ** i for i, cf in enumerate(cash_flows))
            dnpv = sum(-i * cf / (1 + rate) ** (i + 1) for i, cf in enumerate(cash_flows))
            if abs(dnpv) < 1e-12:
                break
            new_rate = rate - npv / dnpv
            if abs(new_rate - rate) < tolerance:
                return new_rate
            rate = new_rate
            if rate < -1:
                rate = -0.5
            if rate > 100:
                rate = 10.0
        npv_final = sum(cf / (1 + rate) ** i for i, cf in enumerate(cash_flows))
        if abs(npv_final) < 0.01:
            return rate
        return None

    @staticmethod
    def _compute_npv(cash_flows: List[float], rate: float) -> float:
        """Compute NPV at a given rate."""
        return sum(cf / (1 + rate) ** i for i, cf in enumerate(cash_flows))

    def health_check(self) -> bool:
        return True

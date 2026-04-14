# Author: Quadri Atharu
"""Analysis routes — financial ratios, trend analysis, peer comparison, DCF valuation."""

from __future__ import annotations

from typing import Any, Dict, List, Optional

from fastapi import APIRouter, HTTPException, status

from ...financial_analysis.profitability_ratios import ProfitabilityRatiosEngine
from ...financial_analysis.liquidity_ratios import LiquidityRatiosEngine
from ...financial_analysis.leverage_ratios import LeverageRatiosEngine
from ...financial_analysis.efficiency_ratios import EfficiencyRatiosEngine
from ...financial_analysis.trend_analysis import TrendAnalysisEngine
from ...financial_analysis.peer_comparison import PeerComparisonEngine
from ...valuation.dcf import DcfEngine

router = APIRouter(prefix="/analysis", tags=["Analysis"])

_profitability = ProfitabilityRatiosEngine()
_liquidity = LiquidityRatiosEngine()
_leverage = LeverageRatiosEngine()
_efficiency = EfficiencyRatiosEngine()
_trend = TrendAnalysisEngine()
_peer = PeerComparisonEngine()
_dcf = DcfEngine()


@router.post("/ratios")
async def compute_all_ratios(body: Dict[str, Any]) -> Dict[str, Any]:
    """Compute all financial ratio categories: profitability, liquidity, leverage, efficiency."""
    try:
        profitability = _profitability.compute_all(body)
        liquidity = _liquidity.compute_all(body)
        leverage = _leverage.compute_all(body)
        efficiency = _efficiency.compute_all(body)
        return {
            "status": "success",
            "ratios": {
                "profitability": profitability,
                "liquidity": liquidity,
                "leverage": leverage,
                "efficiency": efficiency,
            },
        }
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))


@router.post("/trend")
async def compute_trend(body: Dict[str, Any]) -> Dict[str, Any]:
    """Compute multi-period trend analysis for one or more metrics."""
    data_list = body.get("data")
    periods = body.get("periods")
    if not data_list and not periods:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="data or periods is required")
    try:
        if periods:
            trend_data = {
                "periods": periods,
                "metric": body.get("metric", "revenue"),
                "metrics": body.get("metrics"),
            }
            if trend_data.get("metrics"):
                result = _trend.analyze_multiple_metrics(trend_data)
            else:
                result = _trend.analyze_trend(trend_data)
        else:
            result = _trend.analyze_trend({
                "periods": data_list,
                "metric": body.get("metric", "revenue"),
            })
        return {"status": "success", "trend": result}
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))


@router.post("/peer-comparison")
async def compare_to_peers(body: Dict[str, Any]) -> Dict[str, Any]:
    """Compare a company's financials against peer benchmarks."""
    company_ratios = body.get("company_ratios")
    benchmarks = body.get("benchmarks")
    if not company_ratios:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="company_ratios is required")
    try:
        data = {
            "company": company_ratios,
            "peers": benchmarks if isinstance(benchmarks, list) else [],
            "metrics": body.get("metrics", ["revenue", "net_margin", "roe", "current_ratio", "debt_to_equity"]),
        }
        result = _peer.compare_against_peers(data)
        return {"status": "success", "comparison": result}
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))


@router.post("/valuation")
async def compute_dcf_valuation(body: Dict[str, Any]) -> Dict[str, Any]:
    """Compute a DCF valuation with terminal value."""
    cash_flows = body.get("cash_flows")
    discount_rate = body.get("discount_rate")
    if not cash_flows or discount_rate is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="cash_flows and discount_rate are required")
    try:
        data = {
            "cash_flows": cash_flows,
            "discount_rate": float(discount_rate),
            "terminal_growth_rate": float(body.get("growth_rate", 0.03)),
            "terminal_value_method": body.get("terminal_value_method", "gordon_growth"),
            "net_debt": float(body.get("net_debt", 0)),
            "shares_outstanding": float(body.get("shares_outstanding", 1)),
            "exit_multiple": body.get("exit_multiple"),
        }
        result = _dcf.compute_dcf(data)
        return {"status": "success", "valuation": result}
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))

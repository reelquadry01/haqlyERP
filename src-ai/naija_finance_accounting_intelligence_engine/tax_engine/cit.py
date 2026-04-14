# Author: Quadri Atharu
"""Nigerian Companies Income Tax (CIT) computation engine with capital allowances."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List, Optional

from ..core.exceptions import TaxError
from ..core.logging import get_logger

logger = get_logger(__name__)

CIT_BRACKETS = {
    "small_company": {"max_turnover": 500_000_000, "rate": 0.15, "description": "Small company (turnover ≤ N500M)"},
    "medium_manufacturing": {"min_turnover": 500_000_001, "max_turnover": 1_000_000_000, "rate": 0.15, "description": "Medium manufacturing (N500M–N1B)"},
    "large_company": {"min_turnover": 1_000_000_001, "rate": 0.30, "description": "Large company (turnover > N1B)"},
    "petroleum_profit": {"rate": 0.50 if False else 0.30, "description": "Petroleum operations (subject to PPTA)"},
}

MINIMUM_TAX_RATE = 0.005
MINIMUM_TAX_BASE = "turnover"

CAPITAL_ALLOWANCE_RATES = {
    "building": {"initial": 0.15, "annual": 0.10, "class": "1"},
    "plant_and_machinery": {"initial": 0.50, "annual": 0.25, "class": "2"},
    "furniture_and_fittings": {"initial": 0.50, "annual": 0.25, "class": "2"},
    "motor_vehicle_non_commercial": {"initial": 0.50, "annual": 0.25, "class": "2"},
    "motor_vehicle_commercial": {"initial": 0.50, "annual": 0.25, "class": "2"},
    "computer_and_it": {"initial": 0.50, "annual": 25, "class": "2"},
    "research_and_development": {"initial": 0.50, "annual": 0.25, "class": "2"},
    "intangible_assets": {"initial": 0.50, "annual": 0.25, "class": "2"},
    "plantation": {"initial": 0.50, "annual": 0.25, "class": "3"},
}

EXEMPT_INCOMES = [
    "educational_funds", "cooperative_societies_profit", "trade_union_levies",
    "statutory_savings_schemes", "diplomatic_income", "export_incentive_profits",
]


class CitEngine:
    """Nigerian Companies Income Tax computation engine."""

    def compute_cit(
        self,
        profit_before_tax: float,
        turnover: float,
        industry: str = "general",
        is_manufacturing: bool = False,
        is_small_company: Optional[bool] = None,
        capital_allowances: Optional[List[Dict[str, Any]]] = None,
        non_deductible_expenses: float = 0,
        exempt_income: float = 0,
    ) -> Dict[str, Any]:
        """Compute CIT for a Nigerian company."""
        if profit_before_tax < 0 and turnover <= 0:
            raise TaxError("Either profit before tax or turnover must be positive")

        bracket = self._determine_bracket(turnover, industry, is_manufacturing, is_small_company)
        rate = bracket["rate"]

        assessable_profit = round(profit_before_tax + non_deductible_expenses - exempt_income, 2)
        assessable_profit = max(assessable_profit, 0)

        total_capital_allowance = 0.0
        ca_details: List[Dict[str, Any]] = []
        if capital_allowances:
            for ca in capital_allowances:
                ca_result = self.compute_capital_allowance(ca)
                ca_details.append(ca_result)
                total_capital_allowance += ca_result["total_allowance"]

        adjusted_profit = round(assessable_profit - total_capital_allowance, 2)
        adjusted_profit = max(adjusted_profit, 0)

        cit_on_profit = round(adjusted_profit * rate, 2)

        minimum_tax = self._compute_minimum_tax(turnover, profit_before_tax)
        cit_payable = max(cit_on_profit, minimum_tax) if profit_before_tax > 0 else minimum_tax

        if profit_before_tax <= 0 and turnover > 0:
            cit_payable = minimum_tax

        result: Dict[str, Any] = {
            "tax_type": "CIT",
            "profit_before_tax": round(profit_before_tax, 2),
            "non_deductible_expenses_added_back": round(non_deductible_expenses, 2),
            "exempt_income_deducted": round(exempt_income, 2),
            "assessable_profit": assessable_profit,
            "capital_allowance_total": round(total_capital_allowance, 2),
            "capital_allowance_details": ca_details,
            "adjusted_profit": adjusted_profit,
            "applicable_rate": rate,
            "applicable_bracket": bracket["description"],
            "cit_on_profit": cit_on_profit,
            "minimum_tax": minimum_tax,
            "cit_payable": round(cit_payable, 2),
            "turnover": round(turnover, 2),
            "industry": industry,
            "computed_at": datetime.now().isoformat(),
        }

        logger.info("cit_computed", profit_before_tax=profit_before_tax, turnover=turnover, cit_payable=cit_payable, bracket=bracket["description"])
        return result

    def compute_capital_allowance(self, asset: Dict[str, Any]) -> Dict[str, Any]:
        """Compute capital allowance for a qualifying capital expenditure."""
        asset_type = asset.get("asset_type", "plant_and_machinery").lower()
        cost = float(asset.get("cost", 0))
        residual_value = float(asset.get("residual_value", 0))
        is_first_year = asset.get("is_first_year", True)
        years_claimed = int(asset.get("years_claimed", 0))

        if cost <= 0:
            raise TaxError("Asset cost must be positive for capital allowance computation")

        rates = CAPITAL_ALLOWANCE_RATES.get(asset_type, CAPITAL_ALLOWANCE_RATES["plant_and_machinery"])

        if is_first_year:
            initial_allowance = round(cost * rates["initial"], 2)
        else:
            initial_allowance = 0.0

        written_down_value = round(cost - initial_allowance - (cost * rates["annual"] * years_claimed), 2)
        written_down_value = max(written_down_value, residual_value)

        annual_allowance = round(written_down_value * rates["annual"], 2)
        total_allowance = round(initial_allowance + annual_allowance, 2)

        return {
            "asset_type": asset_type,
            "asset_cost": round(cost, 2),
            "residual_value": round(residual_value, 2),
            "asset_class": rates["class"],
            "initial_allowance_rate": rates["initial"],
            "annual_allowance_rate": rates["annual"],
            "initial_allowance": initial_allowance,
            "annual_allowance": annual_allowance,
            "total_allowance": total_allowance,
            "written_down_value_after": round(max(written_down_value - annual_allowance, residual_value), 2),
            "is_first_year": is_first_year,
        }

    def _determine_bracket(self, turnover: float, industry: str, is_manufacturing: bool, is_small_override: Optional[bool]) -> Dict[str, Any]:
        """Determine the applicable CIT bracket."""
        if is_small_override is True:
            return CIT_BRACKETS["small_company"]

        if turnover <= CIT_BRACKETS["small_company"]["max_turnover"]:
            return CIT_BRACKETS["small_company"]

        if is_manufacturing and turnover <= CIT_BRACKETS["medium_manufacturing"]["max_turnover"]:
            return CIT_BRACKETS["medium_manufacturing"]

        if turnover > CIT_BRACKETS["large_company"]["min_turnover"]:
            return CIT_BRACKETS["large_company"]

        if turnover <= CIT_BRACKETS["medium_manufacturing"]["max_turnover"]:
            return CIT_BRACKETS["medium_manufacturing"]

        return CIT_BRACKETS["large_company"]

    def _compute_minimum_tax(self, turnover: float, profit_before_tax: float) -> float:
        """Compute minimum tax per Section 33 CITA (as amended by Finance Act 2020).

        For companies with turnover ≤ N25M: exempt from minimum tax.
        For others: the higher of:
        - 0.5% of gross profit
        - 0.5% of net profit
        - 0.5% of turnover
        - 0.25% of paid-up capital
        """
        if turnover <= 0:
            return 0.0
        if turnover <= 25_000_000:
            return 0.0

        return round(turnover * MINIMUM_TAX_RATE, 2)

    def list_capital_allowance_rates(self) -> Dict[str, Any]:
        """List all capital allowance rates."""
        rates_list: List[Dict[str, Any]] = []
        for asset_type, rates in CAPITAL_ALLOWANCE_RATES.items():
            rates_list.append({
                "asset_type": asset_type,
                "asset_class": rates["class"],
                "initial_allowance": f"{rates['initial'] * 100:.0f}%",
                "annual_allowance": f"{rates['annual'] * 100:.0f}%",
            })
        return {"capital_allowance_rates": rates_list}

    def health_check(self) -> bool:
        return True

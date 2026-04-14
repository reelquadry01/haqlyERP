# Author: Quadri Atharu
"""Macroeconomic module — inflation, interest rates, GDP, FX volatility, scenario analysis."""

from .inflation_rates import InflationRatesEngine
from .interest_rates import InterestRatesEngine
from .gdp_trends import GdpTrendsEngine
from .fx_volatility import FxVolatilityEngine
from .scenario_analysis import ScenarioAnalysisEngine

__all__ = ["InflationRatesEngine", "InterestRatesEngine", "GdpTrendsEngine", "FxVolatilityEngine", "ScenarioAnalysisEngine"]

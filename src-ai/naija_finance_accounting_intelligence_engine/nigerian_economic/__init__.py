# Author: Quadri Atharu
"""Nigerian Economic module — fuel prices, FX restrictions, import dependency, inflation pressure, public sector."""

from .fuel_price import FuelPriceEngine
from .fx_restrictions import FxRestrictionsEngine
from .import_dependency import ImportDependencyEngine
from .inflation_pressure import InflationPressureEngine
from .public_sector import PublicSectorEngine

__all__ = ["FuelPriceEngine", "FxRestrictionsEngine", "ImportDependencyEngine", "InflationPressureEngine", "PublicSectorEngine"]

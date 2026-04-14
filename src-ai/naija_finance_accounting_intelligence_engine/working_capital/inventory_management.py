# Author: Quadri Atharu
"""Inventory management, valuation, and reorder point engine."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.exceptions import AccountingError
from ..core.logging import get_logger

logger = get_logger(__name__)


class InventoryManagementEngine:
    """Inventory valuation and reorder point engine."""

    def compute_inventory_valuation(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute inventory valuation using specified method."""
        method = data.get("method", "weighted_average").lower()
        items: List[Dict[str, Any]] = data.get("items", [])

        if not items:
            return {"total_inventory_value": 0.0, "items": []}

        if method == "fifo":
            return self._fifo_valuation(items)
        elif method == "lifo":
            return self._lifo_valuation(items)
        elif method == "weighted_average":
            return self._weighted_average_valuation(items)
        elif method == "specific_identification":
            return self._specific_identification_valuation(items)
        else:
            raise AccountingError(f"Unknown inventory valuation method: {method}")

    def compute_reorder_point(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute reorder point and safety stock for an item."""
        daily_demand_avg = float(data.get("daily_demand_avg", 0))
        lead_time_days = int(data.get("lead_time_days", 14))
        daily_demand_std = float(data.get("daily_demand_std", 0))
        service_level = float(data.get("service_level", 0.95))
        item_name = data.get("item_name", "")

        from math import erf, sqrt
        z = sqrt(2) * erf(service_level - 0.5) if service_level > 0.5 else 0
        z_approx = {0.90: 1.28, 0.95: 1.65, 0.98: 2.05, 0.99: 2.33}.get(service_level, 1.65)

        safety_stock = round(z_approx * daily_demand_std * (lead_time_days ** 0.5), 2)
        reorder_point = round(daily_demand_avg * lead_time_days + safety_stock, 2)

        return {
            "item_name": item_name,
            "daily_demand_avg": round(daily_demand_avg, 2),
            "lead_time_days": lead_time_days,
            "daily_demand_std": round(daily_demand_std, 2),
            "service_level": service_level,
            "z_score": z_approx,
            "safety_stock": safety_stock,
            "reorder_point": reorder_point,
            "computed_at": datetime.now().isoformat(),
        }

    def compute_inventory_turnover(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute inventory turnover ratio and days inventory outstanding."""
        cogs = float(data.get("cogs", 0))
        avg_inventory = float(data.get("average_inventory", 0))
        period_days = int(data.get("period_days", 365))

        if avg_inventory <= 0:
            return {"turnover": None, "message": "Average inventory must be positive"}

        turnover = round(cogs / avg_inventory, 2)
        dio = round(period_days / turnover, 2) if turnover > 0 else None

        if dio is not None:
            if dio <= 30:
                health = "excellent"
            elif dio <= 60:
                health = "good"
            elif dio <= 90:
                health = "adequate"
            else:
                health = "slow_moving"
        else:
            health = "unknown"

        return {
            "cogs": round(cogs, 2),
            "average_inventory": round(avg_inventory, 2),
            "turnover_ratio": turnover,
            "days_inventory_outstanding": dio,
            "health": health,
        }

    def detect_slow_moving_inventory(self, items: List[Dict[str, Any]], threshold_days: int = 90) -> Dict[str, Any]:
        """Detect slow-moving or obsolete inventory items."""
        slow_moving: List[Dict[str, Any]] = []
        total_slow_value = 0.0

        for item in items:
            last_sold_days = int(item.get("days_since_last_sale", 0))
            quantity = float(item.get("quantity", 0))
            unit_cost = float(item.get("unit_cost", 0))
            value = round(quantity * unit_cost, 2)

            if last_sold_days >= threshold_days:
                provision_rate = 0.25 if last_sold_days < 180 else (0.50 if last_sold_days < 365 else 0.75)
                slow_moving.append({
                    "item": item.get("name", ""),
                    "quantity": quantity,
                    "unit_cost": unit_cost,
                    "total_value": value,
                    "days_since_last_sale": last_sold_days,
                    "recommended_provision_rate": provision_rate,
                    "recommended_provision": round(value * provision_rate, 2),
                })
                total_slow_value += value

        return {
            "slow_moving_items": slow_moving,
            "count": len(slow_moving),
            "total_slow_moving_value": round(total_slow_value, 2),
            "total_recommended_provision": round(sum(s["recommended_provision"] for s in slow_moving), 2),
        }

    def _fifo_valuation(self, items: List[Dict[str, Any]]) -> Dict[str, Any]:
        """FIFO inventory valuation."""
        valued: List[Dict[str, Any]] = []
        total = 0.0

        for item in items:
            layers = item.get("cost_layers", [{"quantity": item.get("quantity", 0), "unit_cost": item.get("unit_cost", 0)}])
            remaining_qty = float(item.get("quantity", 0))
            value = 0.0

            for layer in layers:
                layer_qty = float(layer.get("quantity", 0))
                layer_cost = float(layer.get("unit_cost", 0))
                used = min(layer_qty, remaining_qty)
                value += used * layer_cost
                remaining_qty -= used
                if remaining_qty <= 0:
                    break

            valued.append({"item": item.get("name", ""), "quantity": float(item.get("quantity", 0)), "value": round(value, 2)})
            total += value

        return {"method": "FIFO", "items": valued, "total_inventory_value": round(total, 2)}

    def _lifo_valuation(self, items: List[Dict[str, Any]]) -> Dict[str, Any]:
        """LIFO inventory valuation."""
        valued: List[Dict[str, Any]] = []
        total = 0.0

        for item in items:
            layers = list(reversed(item.get("cost_layers", [{"quantity": item.get("quantity", 0), "unit_cost": item.get("unit_cost", 0)}])))
            remaining_qty = float(item.get("quantity", 0))
            value = 0.0

            for layer in layers:
                layer_qty = float(layer.get("quantity", 0))
                layer_cost = float(layer.get("unit_cost", 0))
                used = min(layer_qty, remaining_qty)
                value += used * layer_cost
                remaining_qty -= used
                if remaining_qty <= 0:
                    break

            valued.append({"item": item.get("name", ""), "quantity": float(item.get("quantity", 0)), "value": round(value, 2)})
            total += value

        return {"method": "LIFO", "items": valued, "total_inventory_value": round(total, 2)}

    def _weighted_average_valuation(self, items: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Weighted average inventory valuation."""
        valued: List[Dict[str, Any]] = []
        total = 0.0

        for item in items:
            qty = float(item.get("quantity", 0))
            unit_cost = float(item.get("unit_cost", 0))
            value = round(qty * unit_cost, 2)
            valued.append({"item": item.get("name", ""), "quantity": qty, "unit_cost": unit_cost, "value": value})
            total += value

        return {"method": "weighted_average", "items": valued, "total_inventory_value": round(total, 2)}

    def _specific_identification_valuation(self, items: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Specific identification inventory valuation."""
        valued: List[Dict[str, Any]] = []
        total = 0.0

        for item in items:
            qty = float(item.get("quantity", 0))
            total_cost = float(item.get("total_cost", qty * float(item.get("unit_cost", 0))))
            valued.append({"item": item.get("name", ""), "quantity": qty, "total_cost": round(total_cost, 2)})
            total += total_cost

        return {"method": "specific_identification", "items": valued, "total_inventory_value": round(total, 2)}

    def health_check(self) -> bool:
        return True

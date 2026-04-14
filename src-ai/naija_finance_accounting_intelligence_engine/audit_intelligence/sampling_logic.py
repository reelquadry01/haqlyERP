# Author: Quadri Atharu
"""Statistical sampling engine for audit procedures."""

from __future__ import annotations

import random
import math
from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)


class SamplingLogicEngine:
    """Statistical sampling engine for audit procedures."""

    def random_sample(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Simple random sampling."""
        population: List[Dict[str, Any]] = data.get("population", [])
        sample_size = int(data.get("sample_size", min(25, len(population))))

        if sample_size > len(population):
            sample_size = len(population)

        sampled = random.sample(population, sample_size)
        return {"method": "random", "population_size": len(population), "sample_size": sample_size, "sample": sampled, "sampling_rate": round(sample_size / len(population), 4) if population else 0}

    def systematic_sample(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Systematic (interval) sampling."""
        population: List[Dict[str, Any]] = data.get("population", [])
        sample_size = int(data.get("sample_size", min(25, len(population))))
        start = int(data.get("random_start", 0))

        if not population or sample_size <= 0:
            return {"method": "systematic", "sample": [], "population_size": 0}

        interval = max(len(population) // sample_size, 1)
        start = start % interval
        sampled = [population[i] for i in range(start, len(population), interval)][:sample_size]

        return {"method": "systematic", "population_size": len(population), "sample_size": len(sampled), "interval": interval, "random_start": start, "sample": sampled}

    def stratified_sample(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Stratified sampling based on a key field."""
        population: List[Dict[str, Any]] = data.get("population", [])
        sample_size = int(data.get("sample_size", 25))
        strata_key = data.get("strata_key", "type")

        strata: Dict[str, List[Dict[str, Any]]] = {}
        for item in population:
            key = str(item.get(strata_key, "unknown"))
            if key not in strata:
                strata[key] = []
            strata[key].append(item)

        total = len(population)
        sampled: List[Dict[str, Any]] = []
        details: Dict[str, Any] = {}

        for key, items in strata.items():
            proportion = len(items) / total if total > 0 else 0
            stratum_sample_size = max(1, round(sample_size * proportion))
            stratum_sample = random.sample(items, min(stratum_sample_size, len(items)))
            sampled.extend(stratum_sample)
            details[key] = {"population": len(items), "sampled": len(stratum_sample)}

        return {"method": "stratified", "population_size": total, "sample_size": len(sampled), "strata_key": strata_key, "strata_details": details, "sample": sampled}

    def monetary_unit_sample(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Monetary unit sampling (MUS) for audit testing."""
        population: List[Dict[str, Any]] = data.get("population", [])
        confidence_level = float(data.get("confidence_level", 0.95))
        tolerable_misstatement = float(data.get("tolerable_misstatement", 0))
        value_key = data.get("value_key", "amount")

        if not population:
            return {"method": "monetary_unit", "sample": [], "population_size": 0}

        total_value = sum(float(item.get(value_key, 0)) for item in population)
        if total_value <= 0 or tolerable_misstatement <= 0:
            return {"method": "monetary_unit", "sample": [], "population_size": len(population), "message": "Invalid total value or tolerable misstatement"}

        sampling_interval = round(tolerable_misstatement / 2, 2)

        random_start = random.uniform(0, sampling_interval)
        selected: List[Dict[str, Any]] = []
        cumulative = 0.0

        for item in population:
            item_value = float(item.get(value_key, 0))
            cumulative += item_value
            while cumulative >= random_start and item_value > 0:
                selected.append(item)
                random_start += sampling_interval

        return {
            "method": "monetary_unit",
            "population_size": len(population),
            "total_book_value": round(total_value, 2),
            "tolerable_misstatement": tolerable_misstatement,
            "sampling_interval": sampling_interval,
            "sample_size": len(selected),
            "sample": selected,
        }

    def compute_sample_size(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute statistically valid sample size."""
        population_size = int(data.get("population_size", 1000))
        confidence_level = float(data.get("confidence_level", 0.95))
        margin_of_error = float(data.get("margin_of_error", 0.05))
        expected_deviation_rate = float(data.get("expected_deviation_rate", 0.02))

        z_scores = {0.90: 1.645, 0.95: 1.96, 0.99: 2.576}
        z = z_scores.get(confidence_level, 1.96)
        p = expected_deviation_rate
        q = 1 - p

        n_infinite = round((z ** 2 * p * q) / margin_of_error ** 2, 0)
        n_adjusted = round(n_infinite / (1 + (n_infinite - 1) / population_size), 0)

        return {
            "population_size": population_size,
            "confidence_level": confidence_level,
            "margin_of_error": margin_of_error,
            "expected_deviation_rate": expected_deviation_rate,
            "z_score": z,
            "sample_size_infinite": int(n_infinite),
            "sample_size_adjusted": int(n_adjusted),
        }

    def health_check(self) -> bool:
        return True

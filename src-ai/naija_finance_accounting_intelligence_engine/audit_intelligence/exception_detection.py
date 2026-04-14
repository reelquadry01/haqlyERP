# Author: Quadri Atharu
"""Exception detection engine — Benford's law analysis and duplicate detection."""

from __future__ import annotations

import math
from collections import Counter
from datetime import datetime
from typing import Any, Dict, List, Tuple

from ..core.logging import get_logger

logger = get_logger(__name__)

BENFORD_EXPECTED = {d: round(math.log10(1 + 1 / d), 4) for d in range(1, 10)}


class ExceptionDetectionEngine:
    """Exception detection engine with Benford's law and duplicate detection."""

    def benford_analysis(self, amounts: List[float]) -> Dict[str, Any]:
        """Perform Benford's Law analysis on a list of amounts."""
        if not amounts:
            return {"message": "No amounts provided for Benford analysis"}

        first_digits: List[int] = []
        for amt in amounts:
            abs_amt = abs(amt)
            if abs_amt == 0:
                continue
            while abs_amt >= 10:
                abs_amt /= 10
            while abs_amt < 1 and abs_amt > 0:
                abs_amt *= 10
            first_digits.append(int(abs_amt))

        if not first_digits:
            return {"message": "No non-zero amounts for Benford analysis"}

        total = len(first_digits)
        observed: Dict[int, float] = {}
        for d in range(1, 10):
            observed[d] = round(first_digits.count(d) / total, 4)

        deviations: Dict[int, Dict[str, Any]] = {}
        flagged_digits: List[Dict[str, Any]] = []

        for d in range(1, 10):
            expected = BENFORD_EXPECTED[d]
            actual = observed.get(d, 0)
            deviation = round(actual - expected, 4)
            deviation_pct = round(deviation / expected * 100, 2) if expected > 0 else 0

            deviations[d] = {
                "expected": expected,
                "observed": actual,
                "deviation": deviation,
                "deviation_pct": deviation_pct,
            }

            if abs(deviation_pct) > 15:
                flagged_digits.append({
                    "digit": d,
                    "expected_pct": round(expected * 100, 2),
                    "observed_pct": round(actual * 100, 2),
                    "deviation_pct": deviation_pct,
                    "severity": "high" if abs(deviation_pct) > 30 else "medium",
                })

        chi_squared = sum((observed.get(d, 0) - BENFORD_EXPECTED[d]) ** 2 / BENFORD_EXPECTED[d] for d in range(1, 10)) * total

        return {
            "total_amounts_analyzed": total,
            "observed_distribution": observed,
            "expected_distribution": BENFORD_EXPECTED,
            "deviations": deviations,
            "flagged_digits": flagged_digits,
            "flagged_count": len(flagged_digits),
            "chi_squared_statistic": round(chi_squared, 2),
            "potential_fraud_indicator": len(flagged_digits) > 0,
            "computed_at": datetime.now().isoformat(),
        }

    def detect_duplicates(self, records: List[Dict[str, Any]], key_fields: List[str] | None = None) -> Dict[str, Any]:
        """Detect duplicate records based on key fields."""
        if key_fields is None:
            key_fields = ["reference", "amount"]

        seen: Dict[str, List[Dict[str, Any]]] = {}
        for record in records:
            key = "|".join(str(record.get(f, "")) for f in key_fields)
            if key not in seen:
                seen[key] = []
            seen[key].append(record)

        duplicates: List[Dict[str, Any]] = []
        for key, records_list in seen.items():
            if len(records_list) > 1:
                duplicates.append({"key": key, "count": len(records_list), "records": records_list})

        return {
            "total_records": len(records),
            "unique_keys": len(seen),
            "duplicate_groups": len(duplicates),
            "total_duplicate_records": sum(d["count"] for d in duplicates),
            "duplicates": duplicates,
            "key_fields": key_fields,
        }

    def detect_unusual_amounts(self, amounts: List[float], threshold_std: float = 2.5) -> Dict[str, Any]:
        """Detect unusual amounts using statistical outlier detection."""
        if not amounts:
            return {"message": "No amounts provided"}

        valid = [a for a in amounts if a != 0]
        if not valid:
            return {"message": "No non-zero amounts"}

        mean = sum(valid) / len(valid)
        std = (sum((a - mean) ** 2 for a in valid) / (len(valid) - 1)) ** 0.5 if len(valid) > 1 else 0

        upper = mean + threshold_std * std
        lower = mean - threshold_std * std

        outliers = [{"amount": a, "z_score": round((a - mean) / std, 2) if std > 0 else 0, "direction": "high" if a > upper else "low"} for a in valid if a > upper or a < lower]

        return {
            "total_amounts": len(valid),
            "mean": round(mean, 2),
            "std_dev": round(std, 2),
            "threshold_std": threshold_std,
            "upper_threshold": round(upper, 2),
            "lower_threshold": round(lower, 2),
            "outlier_count": len(outliers),
            "outliers": outliers,
        }

    def detect_late_entries(self, entries: List[Dict[str, Any]], threshold_days: int = 5) -> Dict[str, Any]:
        """Detect journal entries posted significantly after period end."""
        late: List[Dict[str, Any]] = []

        for entry in entries:
            entry_date = entry.get("entry_date", "")
            period_end = entry.get("period_end", "")
            if not entry_date or not period_end:
                continue

            try:
                ed = datetime.fromisoformat(entry_date)
                pe = datetime.fromisoformat(period_end)
                days_late = (ed - pe).days
                if days_late > threshold_days:
                    late.append({**entry, "days_late": days_late})
            except (ValueError, TypeError):
                continue

        return {
            "total_entries_scanned": len(entries),
            "late_entry_count": len(late),
            "threshold_days": threshold_days,
            "late_entries": late,
        }

    def health_check(self) -> bool:
        return True

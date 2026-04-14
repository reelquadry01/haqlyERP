# Author: Quadri Atharu
"""Audit trail generation and suspicious pattern detection.

Provides comprehensive audit trail recording, trail generation,
and pattern-based anomaly detection for HAQLY ERP financial operations.
"""

from __future__ import annotations

import uuid
from collections import Counter, defaultdict
from datetime import datetime, timedelta
from typing import Any, Dict, List, Optional, Tuple

from ..core.logging import get_logger

logger = get_logger(__name__)

_audit_store: List[Dict[str, Any]] = []

SUSPICIOUS_THRESHOLDS: Dict[str, Any] = {
    "max_entries_per_user_per_day": 50,
    "max_amount_single_entry": 100_000_000,
    "max_voids_per_user_per_day": 5,
    "max_reversals_per_user_per_day": 5,
    "after_hours_start": 20,
    "after_hours_end": 6,
    "weekend_days": {5, 6},
    "max_same_amount_repeats": 10,
}


def record_action(
    user_id: str,
    action: str,
    entity_type: str,
    entity_id: str,
    details: Optional[Dict[str, Any]] = None,
    company_id: Optional[str] = None,
    ip_address: Optional[str] = None,
    session_id: Optional[str] = None,
) -> Dict[str, Any]:
    """Record a user action in the audit trail.

    Returns the created audit record with unique ID and timestamp.
    """
    record: Dict[str, Any] = {
        "id": str(uuid.uuid4()),
        "user_id": user_id,
        "action": action,
        "entity_type": entity_type,
        "entity_id": entity_id,
        "details": details or {},
        "company_id": company_id,
        "ip_address": ip_address,
        "session_id": session_id,
        "timestamp": datetime.now().isoformat(),
        "date": datetime.now().strftime("%Y-%m-%d"),
        "hour": datetime.now().hour,
        "day_of_week": datetime.now().weekday(),
    }

    _audit_store.append(record)
    logger.info(
        "action_recorded",
        user_id=user_id,
        action=action,
        entity_type=entity_type,
        entity_id=entity_id,
    )

    return record


def generate_audit_trail(
    company_id: Optional[str] = None,
    user_id: Optional[str] = None,
    entity_type: Optional[str] = None,
    action: Optional[str] = None,
    date_from: Optional[str] = None,
    date_to: Optional[str] = None,
    limit: int = 1000,
) -> Dict[str, Any]:
    """Generate a filtered audit trail report.

    Supports filtering by company, user, entity type, action, and date range.
    """
    entries = list(_audit_store)

    if company_id:
        entries = [e for e in entries if e.get("company_id") == company_id]
    if user_id:
        entries = [e for e in entries if e.get("user_id") == user_id]
    if entity_type:
        entries = [e for e in entries if e.get("entity_type") == entity_type]
    if action:
        entries = [e for e in entries if e.get("action") == action]
    if date_from:
        entries = [e for e in entries if e.get("date", "") >= date_from]
    if date_to:
        entries = [e for e in entries if e.get("date", "") <= date_to]

    entries.sort(key=lambda e: e.get("timestamp", ""), reverse=True)
    total = len(entries)
    entries = entries[:limit]

    action_counts = Counter(e.get("action", "") for e in entries)
    user_counts = Counter(e.get("user_id", "") for e in entries)
    entity_counts = Counter(e.get("entity_type", "") for e in entries)

    return {
        "total_records": total,
        "returned_count": len(entries),
        "filters": {
            "company_id": company_id,
            "user_id": user_id,
            "entity_type": entity_type,
            "action": action,
            "date_from": date_from,
            "date_to": date_to,
        },
        "entries": entries,
        "summary": {
            "by_action": dict(action_counts),
            "by_user": dict(user_counts),
            "by_entity_type": dict(entity_counts),
        },
        "generated_at": datetime.now().isoformat(),
    }


def generate_audit_summary(company_id: Optional[str] = None, days: int = 30) -> Dict[str, Any]:
    """Generate an audit summary report for the last N days."""
    cutoff = (datetime.now() - timedelta(days=days)).isoformat()
    entries = [e for e in _audit_store if e.get("timestamp", "") >= cutoff]
    if company_id:
        entries = [e for e in entries if e.get("company_id") == company_id]

    action_counts = Counter(e.get("action", "") for e in entries)
    user_counts = Counter(e.get("user_id", "") for e in entries)
    daily_counts = Counter(e.get("date", "") for e in entries)

    return {
        "period_days": days,
        "company_id": company_id,
        "total_actions": len(entries),
        "unique_users": len(user_counts),
        "by_action": dict(action_counts.most_common(20)),
        "by_user": dict(user_counts.most_common(20)),
        "daily_activity": dict(sorted(daily_counts.items())),
        "generated_at": datetime.now().isoformat(),
    }


def detect_suspicious_patterns(
    company_id: Optional[str] = None,
    thresholds: Optional[Dict[str, Any]] = None,
) -> Dict[str, Any]:
    """Detect suspicious patterns in financial actions.

    Checks for: unusual volume, after-hours activity, excessive voids/reversals,
    round-number patterns, and same-user SoD violations.
    """
    th = {**SUSPICIOUS_THRESHOLDS, **(thresholds or {})}
    entries = list(_audit_store)
    if company_id:
        entries = [e for e in entries if e.get("company_id") == company_id]

    alerts: List[Dict[str, Any]] = []

    user_day_counts: Dict[str, Counter] = defaultdict(Counter)
    user_action_day: Dict[str, Counter] = defaultdict(Counter)
    amount_repeats: Counter = Counter()

    for e in entries:
        user = e.get("user_id", "unknown")
        date = e.get("date", "")
        user_day_counts[user][date] += 1

        action = e.get("action", "")
        if action in ("VOID", "REVERSE"):
            user_action_day[user][date] += 1

        amt = e.get("details", {}).get("amount")
        if amt is not None:
            amount_repeats[(user, round(float(amt), 2))] += 1

    for user, day_counter in user_day_counts.items():
        for day, count in day_counter.items():
            if count > th["max_entries_per_user_per_day"]:
                alerts.append({
                    "type": "HIGH_VOLUME",
                    "severity": "medium",
                    "user_id": user,
                    "date": day,
                    "count": count,
                    "threshold": th["max_entries_per_user_per_day"],
                    "description": f"User {user} created {count} entries on {day} (threshold: {th['max_entries_per_user_per_day']})",
                })

    for user, action_counter in user_action_day.items():
        for day, count in action_counter.items():
            if count > th["max_voids_per_user_per_day"]:
                alerts.append({
                    "type": "EXCESSIVE_VOIDS_REVERSALS",
                    "severity": "high",
                    "user_id": user,
                    "date": day,
                    "count": count,
                    "threshold": th["max_voids_per_user_per_day"],
                    "description": f"User {user} voided/reversed {count} entries on {day}",
                })

    after_hours = [
        e for e in entries
        if e.get("hour", 12) >= th["after_hours_start"] or e.get("hour", 12) < th["after_hours_end"]
    ]
    if after_hours:
        users_involved = len(set(e.get("user_id") for e in after_hours))
        alerts.append({
            "type": "AFTER_HOURS_ACTIVITY",
            "severity": "low",
            "count": len(after_hours),
            "unique_users": users_involved,
            "description": f"{len(after_hours)} actions recorded after hours by {users_involved} users",
        })

    weekend_entries = [e for e in entries if e.get("day_of_week", 0) in th["weekend_days"]]
    if weekend_entries:
        alerts.append({
            "type": "WEEKEND_ACTIVITY",
            "severity": "low",
            "count": len(weekend_entries),
            "unique_users": len(set(e.get("user_id") for e in weekend_entries)),
            "description": f"{len(weekend_entries)} actions recorded on weekends",
        })

    for (user, amount), count in amount_repeats.items():
        if count > th["max_same_amount_repeats"]:
            alerts.append({
                "type": "REPEATED_SAME_AMOUNT",
                "severity": "medium",
                "user_id": user,
                "amount": amount,
                "count": count,
                "threshold": th["max_same_amount_repeats"],
                "description": f"User {user} posted {count} entries with identical amount {amount}",
            })

    high_amount_entries = [
        e for e in entries
        if (e.get("details", {}).get("amount") or 0) > th["max_amount_single_entry"]
    ]
    if high_amount_entries:
        alerts.append({
            "type": "HIGH_VALUE_TRANSACTION",
            "severity": "high",
            "count": len(high_amount_entries),
            "threshold": th["max_amount_single_entry"],
            "description": f"{len(high_amount_entries)} transactions exceed {th['max_amount_single_entry']:,.0f} NGN",
        })

    return {
        "total_entries_analyzed": len(entries),
        "company_id": company_id,
        "alert_count": len(alerts),
        "alerts": alerts,
        "severity_summary": {
            "low": sum(1 for a in alerts if a.get("severity") == "low"),
            "medium": sum(1 for a in alerts if a.get("severity") == "medium"),
            "high": sum(1 for a in alerts if a.get("severity") == "high"),
            "critical": sum(1 for a in alerts if a.get("severity") == "critical"),
        },
        "thresholds_used": th,
        "analyzed_at": datetime.now().isoformat(),
    }

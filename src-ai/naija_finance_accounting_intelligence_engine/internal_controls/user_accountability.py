# Author: Quadri Atharu
"""User accountability tracking and dormant control flagging.

Tracks user actions, computes activity summaries, and flags
dormant controls and inactive users for HAQLY ERP.
"""

from __future__ import annotations

import uuid
from collections import Counter, defaultdict
from datetime import datetime, timedelta
from typing import Any, Dict, List, Optional

from ..core.logging import get_logger

logger = get_logger(__name__)

_user_action_store: List[Dict[str, Any]] = []


def track_user_action(
    user_id: str,
    action_type: str,
    module: str,
    description: str = "",
    entity_id: Optional[str] = None,
    amount: Optional[float] = None,
    company_id: Optional[str] = None,
    outcome: str = "success",
    ip_address: Optional[str] = None,
) -> Dict[str, Any]:
    """Track a user action for accountability purposes.

    Records the action with full metadata for later analysis and reporting.
    """
    record: Dict[str, Any] = {
        "id": str(uuid.uuid4()),
        "user_id": user_id,
        "action_type": action_type,
        "module": module,
        "description": description,
        "entity_id": entity_id,
        "amount": amount,
        "company_id": company_id,
        "outcome": outcome,
        "ip_address": ip_address,
        "timestamp": datetime.now().isoformat(),
        "date": datetime.now().strftime("%Y-%m-%d"),
    }

    _user_action_store.append(record)

    logger.info(
        "user_action_tracked",
        user_id=user_id,
        action_type=action_type,
        module=module,
        outcome=outcome,
    )

    return record


def compute_user_activity_summary(
    user_id: Optional[str] = None,
    company_id: Optional[str] = None,
    days: int = 30,
) -> Dict[str, Any]:
    """Compute a comprehensive activity summary for one or all users.

    Aggregates action counts, amounts, modules used, and timing patterns.
    """
    cutoff = (datetime.now() - timedelta(days=days)).strftime("%Y-%m-%d")
    entries = [e for e in _user_action_store if e.get("date", "") >= cutoff]

    if user_id:
        entries = [e for e in entries if e.get("user_id") == user_id]
    if company_id:
        entries = [e for e in entries if e.get("company_id") == company_id]

    if user_id:
        return _single_user_summary(user_id, entries, days)

    user_groups: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
    for e in entries:
        user_groups[e.get("user_id", "unknown")].append(e)

    user_summaries = {}
    for uid, user_entries in user_groups.items():
        user_summaries[uid] = _single_user_summary(uid, user_entries, days)

    total_actions = len(entries)
    active_users = len(user_groups)
    avg_actions_per_user = round(total_actions / active_users, 2) if active_users else 0

    module_counts = Counter(e.get("module", "") for e in entries)
    action_type_counts = Counter(e.get("action_type", "") for e in entries)

    return {
        "period_days": days,
        "company_id": company_id,
        "total_actions": total_actions,
        "active_users": active_users,
        "avg_actions_per_user": avg_actions_per_user,
        "by_module": dict(module_counts.most_common()),
        "by_action_type": dict(action_type_counts.most_common()),
        "user_summaries": user_summaries,
        "computed_at": datetime.now().isoformat(),
    }


def flag_dormant_controls(
    company_id: Optional[str] = None,
    dormant_days: int = 30,
    critical_modules: Optional[List[str]] = None,
) -> Dict[str, Any]:
    """Flag dormant controls — users and modules with no recent activity.

    Identifies: inactive users, unused approval workflows, unexercised
    review controls, and modules with no recent activity.
    """
    if critical_modules is None:
        critical_modules = [
            "internal_controls",
            "audit_trail",
            "tax_engine",
            "journal_engine",
            "ledger_engine",
            "approval_workflow",
            "period_close",
            "bank_reconciliation",
        ]

    cutoff = (datetime.now() - timedelta(days=dormant_days)).strftime("%Y-%m-%d")
    recent_entries = [e for e in _user_action_store if e.get("date", "") >= cutoff]
    if company_id:
        recent_entries = [e for e in recent_entries if e.get("company_id") == company_id]

    all_entries = list(_user_action_store)
    if company_id:
        all_entries = [e for e in all_entries if e.get("company_id") == company_id]

    recent_users = {e.get("user_id") for e in recent_entries}
    all_users = {e.get("user_id") for e in all_entries}

    dormant_users = all_users - recent_users

    recent_modules = {e.get("module") for e in recent_entries}
    dormant_modules = set(critical_modules) - recent_modules

    dormant_flags: List[Dict[str, Any]] = []

    for user in dormant_users:
        user_entries = [e for e in all_entries if e.get("user_id") == user]
        last_action = max(e.get("date", "") for e in user_entries) if user_entries else ""
        user_modules = {e.get("module") for e in user_entries}
        had_critical = bool(user_modules & set(critical_modules))

        dormant_flags.append({
            "type": "DORMANT_USER",
            "severity": "high" if had_critical else "medium",
            "user_id": user,
            "last_action_date": last_action,
            "dormant_days": dormant_days,
            "previously_active_modules": sorted(user_modules),
            "had_critical_module_access": had_critical,
            "description": f"User {user} inactive for {dormant_days}+ days" + (" (had critical access)" if had_critical else ""),
        })

    for module in dormant_modules:
        module_entries = [e for e in all_entries if e.get("module") == module]
        last_action = max(e.get("date", "") for e in module_entries) if module_entries else "never"
        is_critical = module in critical_modules

        dormant_flags.append({
            "type": "DORMANT_MODULE",
            "severity": "critical" if is_critical else "low",
            "module": module,
            "last_action_date": last_action,
            "dormant_days": dormant_days,
            "is_critical_module": is_critical,
            "description": f"No activity in {module} for {dormant_days}+ days" + (" — critical control" if is_critical else ""),
        })

    approval_entries = [e for e in recent_entries if e.get("action_type") in ("APPROVE", "REVIEW")]
    approval_users = {e.get("user_id") for e in approval_entries}
    approval_reviewers = {e.get("user_id") for e in approval_entries if e.get("action_type") == "REVIEW"}

    if not approval_reviewers:
        dormant_flags.append({
            "type": "NO_REVIEW_ACTIVITY",
            "severity": "critical",
            "description": f"No review actions recorded in the last {dormant_days} days — review controls may be dormant",
        })

    critical_flags = sum(1 for f in dormant_flags if f.get("severity") == "critical")
    high_flags = sum(1 for f in dormant_flags if f.get("severity") == "high")

    return {
        "company_id": company_id,
        "dormant_threshold_days": dormant_days,
        "total_flags": len(dormant_flags),
        "critical_flags": critical_flags,
        "high_flags": high_flags,
        "dormant_users_count": len(dormant_users),
        "dormant_modules_count": len(dormant_modules),
        "active_users_count": len(recent_users),
        "total_known_users": len(all_users),
        "flags": dormant_flags,
        "critical_modules_checked": critical_modules,
        "flagged_at": datetime.now().isoformat(),
    }


def _single_user_summary(
    user_id: str,
    entries: List[Dict[str, Any]],
    days: int,
) -> Dict[str, Any]:
    """Compute summary for a single user."""
    if not entries:
        return {
            "user_id": user_id,
            "total_actions": 0,
            "period_days": days,
            "status": "inactive",
        }

    action_counts = Counter(e.get("action_type", "") for e in entries)
    module_counts = Counter(e.get("module", "") for e in entries)
    daily_counts = Counter(e.get("date", "") for e in entries)

    amounts = [float(e.get("amount", 0)) for e in entries if e.get("amount") is not None]
    total_amount = round(sum(amounts), 2) if amounts else 0
    avg_amount = round(total_amount / len(amounts), 2) if amounts else 0

    outcomes = Counter(e.get("outcome", "") for e in entries)
    success_rate = round(outcomes.get("success", 0) / len(entries), 4) if entries else 0

    last_action = max(e.get("timestamp", "") for e in entries)

    active_days = len(daily_counts)
    avg_actions_per_day = round(len(entries) / active_days, 2) if active_days else 0

    return {
        "user_id": user_id,
        "total_actions": len(entries),
        "period_days": days,
        "active_days": active_days,
        "avg_actions_per_day": avg_actions_per_day,
        "total_amount_processed": total_amount,
        "avg_amount_per_action": avg_amount,
        "by_action_type": dict(action_counts.most_common()),
        "by_module": dict(module_counts.most_common()),
        "by_date": dict(sorted(daily_counts.items())),
        "success_rate": success_rate,
        "outcomes": dict(outcomes),
        "last_action": last_action,
        "status": "active",
    }

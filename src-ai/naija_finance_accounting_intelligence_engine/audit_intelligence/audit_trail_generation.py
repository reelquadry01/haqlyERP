# Author: Quadri Atharu
"""Audit trail generation engine."""

from __future__ import annotations

import uuid
from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)


class AuditTrailEngine:
    """Full audit trail generation and query engine."""

    def __init__(self) -> None:
        self._entries: List[Dict[str, Any]] = []

    def record_event(self, event: Dict[str, Any]) -> Dict[str, Any]:
        """Record an audit trail event."""
        entry = {
            "id": str(uuid.uuid4()),
            "company_id": event.get("company_id", ""),
            "user_id": event.get("user_id"),
            "action": event.get("action", ""),
            "entity_type": event.get("entity_type", ""),
            "entity_id": event.get("entity_id", ""),
            "old_value": event.get("old_value"),
            "new_value": event.get("new_value"),
            "timestamp": datetime.now().isoformat(),
            "ip_address": event.get("ip_address"),
            "user_agent": event.get("user_agent"),
            "notes": event.get("notes"),
        }
        self._entries.append(entry)
        logger.info("audit_event_recorded", action=entry["action"], entity=entry["entity_type"])
        return entry

    def query_trail(self, filters: Dict[str, Any]) -> Dict[str, Any]:
        """Query audit trail with filters."""
        results = list(self._entries)

        for key, value in filters.items():
            if key == "company_id":
                results = [e for e in results if e.get("company_id") == value]
            elif key == "user_id":
                results = [e for e in results if e.get("user_id") == value]
            elif key == "action":
                results = [e for e in results if e.get("action") == value]
            elif key == "entity_type":
                results = [e for e in results if e.get("entity_type") == value]
            elif key == "entity_id":
                results = [e for e in results if e.get("entity_id") == value]
            elif key == "date_from":
                results = [e for e in results if e.get("timestamp", "") >= value]
            elif key == "date_to":
                results = [e for e in results if e.get("timestamp", "") <= value]

        return {"entries": results, "count": len(results)}

    def generate_trail_report(self, company_id: str, period_start: str, period_end: str) -> Dict[str, Any]:
        """Generate an audit trail report for a period."""
        entries = [e for e in self._entries if e.get("company_id") == company_id and e.get("timestamp", "") >= period_start and e.get("timestamp", "") <= period_end]

        action_counts: Dict[str, int] = {}
        for e in entries:
            action = e.get("action", "UNKNOWN")
            action_counts[action] = action_counts.get(action, 0) + 1

        return {
            "company_id": company_id,
            "period_start": period_start,
            "period_end": period_end,
            "total_events": len(entries),
            "action_breakdown": action_counts,
            "unique_users": len(set(e.get("user_id", "") for e in entries if e.get("user_id"))),
            "entries": entries,
            "generated_at": datetime.now().isoformat(),
        }

    def health_check(self) -> bool:
        return True

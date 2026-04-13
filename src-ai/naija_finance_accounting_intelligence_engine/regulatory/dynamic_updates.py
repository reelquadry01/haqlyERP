"""Dynamic regulatory update framework.

Author: Quadri Atharu

Manages regulatory change detection, versioning, and application
of updates to tax rates, compliance requirements, and policy
parameters. Supports FIRS, CBN, CAC, and other Nigerian regulators.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any
from datetime import datetime


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class RegulatoryUpdate:
    update_id: str
    regulation_type: str
    description: str
    effective_date: str
    old_value: str
    new_value: str
    authority: str
    status: str = "pending"


@dataclass
class VersionRecord:
    version: str
    regulation_type: str
    effective_date: str
    changes: list[RegulatoryUpdate]
    applied_at: str = ""


class RegulatoryUpdateManager:
    """Manages detection, versioning, and application of regulatory changes."""

    def __init__(self) -> None:
        self._updates: dict[str, list[RegulatoryUpdate]] = {}
        self._version_history: dict[str, list[VersionRecord]] = {}
        self._current_versions: dict[str, str] = {}

    def check_for_updates(self, regulation_type: str) -> list[RegulatoryUpdate]:
        """Check for pending regulatory updates of a given type.

        Scans registered update sources for changes in tax rates,
        compliance requirements, or policy parameters.

        Args:
            regulation_type: Type of regulation (e.g. 'tax_rates',
                           'cbn_policy', 'cama', 'ifrs', 'nbs').

        Returns:
            List of RegulatoryUpdate objects pending application.
        """
        pending = [
            u for u in self._updates.get(regulation_type, [])
            if u.status == "pending"
        ]
        return pending

    def apply_update(self, update: RegulatoryUpdate) -> VersionRecord:
        """Apply a regulatory update and record it in version history.

        Marks the update as applied and creates a version record
        for audit trail purposes.

        Args:
            update: The RegulatoryUpdate to apply.

        Returns:
            VersionRecord documenting the applied change.
        """
        update.status = "applied"

        reg_type = update.regulation_type
        if reg_type not in self._updates:
            self._updates[reg_type] = []

        current_version = self._current_versions.get(reg_type, "0.0")
        parts = current_version.split(".")
        new_minor = int(parts[-1]) + 1 if len(parts) > 1 else 1
        new_version = f"{parts[0]}.{new_minor}"

        version_record = VersionRecord(
            version=new_version,
            regulation_type=reg_type,
            effective_date=update.effective_date,
            changes=[update],
            applied_at=datetime.now().isoformat(),
        )

        if reg_type not in self._version_history:
            self._version_history[reg_type] = []
        self._version_history[reg_type].append(version_record)

        self._current_versions[reg_type] = new_version

        return version_record

    def get_version_history(self, regulation_type: str) -> list[VersionRecord]:
        """Get the version history for a regulation type.

        Args:
            regulation_type: The regulation category to query.

        Returns:
            List of VersionRecord objects in chronological order.
        """
        return list(self._version_history.get(regulation_type, []))

    def register_update(self, update: RegulatoryUpdate) -> None:
        """Register a new regulatory update for tracking.

        Args:
            update: The RegulatoryUpdate to register.
        """
        reg_type = update.regulation_type
        if reg_type not in self._updates:
            self._updates[reg_type] = []
        self._updates[reg_type].append(update)

    def get_current_version(self, regulation_type: str) -> str:
        """Get the current version number for a regulation type.

        Args:
            regulation_type: The regulation category.

        Returns:
            Version string (e.g. '1.3').
        """
        return self._current_versions.get(regulation_type, "1.0")

"""Group reporting structure management for multi-entity consolidation.

Author: Quadri Atharu

Defines group structures, determines consolidation scope based on
control and significant influence thresholds, and manages ownership
percentages for Nigerian corporate groups per CAMA 2020 and IFRS 10.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class EntityOwnership:
    entity_name: str
    ownership_pct: Decimal
    has_control: bool
    has_significant_influence: bool


@dataclass
class GroupStructure:
    parent_name: str
    subsidiaries: list[EntityOwnership] = field(default_factory=list)
    associates: list[EntityOwnership] = field(default_factory=list)
    joint_ventures: list[EntityOwnership] = field(default_factory=list)

    def all_entities(self) -> list[str]:
        entities = [self.parent_name]
        for sub in self.subsidiaries:
            entities.append(sub.entity_name)
        for assoc in self.associates:
            entities.append(assoc.entity_name)
        for jv in self.joint_ventures:
            entities.append(jv.entity_name)
        return entities

    def subsidiaries_names(self) -> list[str]:
        return [s.entity_name for s in self.subsidiaries]

    def associates_names(self) -> list[str]:
        return [a.entity_name for a in self.associates]


@dataclass
class ConsolidationScope:
    entities_to_consolidate: list[str]
    entities_to_equity_account: list[str]
    entities_excluded: list[str]
    reason: str


CONTROL_THRESHOLD = Decimal("50")
SIGNIFICANT_INFLUENCE_THRESHOLD = Decimal("20")


def define_group_structure(
    parent: str,
    subsidiaries: list[dict[str, Any]],
    ownership_percentages: dict[str, Decimal],
) -> GroupStructure:
    """Define a group structure by classifying each investee as a subsidiary
    (>=50%), associate (20-49%), or joint venture based on ownership.

    Applies IFRS 10 control criteria and IAS 28 significant influence
    thresholds, relevant for Nigerian groups under CAMA 2020.

    Args:
        parent: Name of the parent entity.
        subsidiaries: List of dicts with 'name' for each investee entity.
        ownership_percentages: Dict mapping entity name to ownership % (0-100).

    Returns:
        GroupStructure with classified subsidiaries, associates, and JVs.
    """
    group = GroupStructure(parent_name=parent)
    control = CONTROL_THRESHOLD
    sig_influence = SIGNIFICANT_INFLUENCE_THRESHOLD

    for sub_info in subsidiaries:
        name = sub_info["name"]
        pct = _d(ownership_percentages.get(name, 0))
        has_control = pct >= control
        has_significant_influence = pct >= sig_influence

        ownership = EntityOwnership(
            entity_name=name,
            ownership_pct=pct,
            has_control=has_control,
            has_significant_influence=has_significant_influence,
        )

        if has_control:
            group.subsidiaries.append(ownership)
        elif pct >= sig_influence and pct < control:
            group.associates.append(ownership)
        else:
            group.joint_ventures.append(ownership)

    return group


def get_consolidation_scope(
    group: GroupStructure,
    period: str,
) -> ConsolidationScope:
    """Determine which entities to include in the consolidation scope
    for a given reporting period.

    Subsidiaries (>=50% ownership) are fully consolidated.
    Associates (20-49%) are equity-accounted.
    Entities below 20% are excluded from consolidation but may
    require disclosure.

    Args:
        group: The GroupStructure defining the group hierarchy.
        period: The reporting period for scope determination.

    Returns:
        ConsolidationScope listing entities for full consolidation,
        equity accounting, and exclusion.
    """
    consolidate = []
    equity_account = []
    excluded = []

    for sub in group.subsidiaries:
        consolidate.append(sub.entity_name)

    for assoc in group.associates:
        equity_account.append(assoc.entity_name)

    for jv in group.joint_ventures:
        if jv.has_significant_influence:
            equity_account.append(jv.entity_name)
        else:
            excluded.append(jv.entity_name)

    return ConsolidationScope(
        entities_to_consolidate=consolidate,
        entities_to_equity_account=equity_account,
        entities_excluded=excluded,
        reason=f"Consolidation scope for period {period} based on IFRS 10 control criteria",
    )


def check_control_threshold(ownership_pct: Decimal) -> bool:
    """Check if the ownership percentage meets the control threshold.

    Under IFRS 10, control requires power over the investee, exposure
    to variable returns, and ability to use power to affect returns.
    Ownership of >=50% typically presumes control. Significant
    influence is presumed at >=20% per IAS 28.

    Args:
        ownership_pct: Ownership percentage (0-100).

    Returns:
        True if ownership >= 50% (control threshold).
    """
    ownership_pct = _d(ownership_pct)
    return ownership_pct >= CONTROL_THRESHOLD

# Author: Quadri Atharu
"""Financial data validation — balance checks, date ranges, account existence, and rule-based validation."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List, Optional

from ..core.exceptions import ValidationError
from ..core.logging import get_logger

logger = get_logger(__name__)


class ValidationResult:
    """Structured validation result with errors, warnings, and metadata."""

    __slots__ = ("is_valid", "errors", "warnings", "checked_at", "rule_count", "passed_count", "failed_count")

    def __init__(
        self,
        is_valid: bool,
        errors: List[str],
        warnings: List[str],
        rule_count: int = 0,
        passed_count: int = 0,
        failed_count: int = 0,
    ) -> None:
        self.is_valid = is_valid
        self.errors = errors
        self.warnings = warnings
        self.checked_at = datetime.now().isoformat()
        self.rule_count = rule_count
        self.passed_count = passed_count
        self.failed_count = failed_count

    def to_dict(self) -> Dict[str, Any]:
        return {
            "is_valid": self.is_valid,
            "errors": self.errors,
            "warnings": self.warnings,
            "checked_at": self.checked_at,
            "rule_count": self.rule_count,
            "passed_count": self.passed_count,
            "failed_count": self.failed_count,
        }

    def __bool__(self) -> bool:
        return self.is_valid


def validate_financial_data(data: Dict[str, Any], rules: List[Dict[str, Any]]) -> ValidationResult:
    errors: List[str] = []
    warnings: List[str] = []
    passed = 0
    failed = 0

    for rule in rules:
        rule_name = rule.get("name", "unnamed_rule")
        rule_type = rule.get("type", "custom")
        field = rule.get("field", "")
        condition = rule.get("condition", "")
        message = rule.get("message", f"Validation failed for rule: {rule_name}")
        severity = rule.get("severity", "error")

        value = _get_nested_value(data, field)
        check_passed = _evaluate_condition(value, condition, rule)

        if check_passed:
            passed += 1
        else:
            failed += 1
            if severity == "warning":
                warnings.append(f"[{rule_name}] {message}")
            else:
                errors.append(f"[{rule_name}] {message}")

    result = ValidationResult(
        is_valid=len(errors) == 0,
        errors=errors,
        warnings=warnings,
        rule_count=len(rules),
        passed_count=passed,
        failed_count=failed,
    )

    if not result.is_valid:
        logger.warning("financial_data_validation_failed", errors=len(errors), warnings=len(warnings))
    else:
        logger.info("financial_data_validation_passed", rules=len(rules))

    return result


def validate_balance(journal: Dict[str, Any]) -> bool:
    lines = journal.get("lines", [])
    if not lines:
        logger.warning("validate_balance_no_lines")
        return False

    total_debit = round(sum(float(line.get("debit", 0)) for line in lines), 2)
    total_credit = round(sum(float(line.get("credit", 0)) for line in lines), 2)
    difference = abs(total_debit - total_credit)
    is_balanced = difference <= 0.01

    if not is_balanced:
        logger.warning(
            "balance_check_failed",
            total_debit=total_debit,
            total_credit=total_credit,
            difference=difference,
        )

    return is_balanced


def validate_date_range(start: str, end: str) -> bool:
    try:
        start_dt = datetime.fromisoformat(start.replace("Z", "+00:00"))
        end_dt = datetime.fromisoformat(end.replace("Z", "+00:00"))
    except (ValueError, TypeError, AttributeError):
        logger.warning("date_range_invalid_format", start=start, end=end)
        return False

    if start_dt > end_dt:
        logger.warning("date_range_start_after_end", start=start, end=end)
        return False

    return True


def validate_account_exists(account_id: str, chart_of_accounts: List[Dict[str, Any]]) -> bool:
    if not account_id:
        return False
    for account in chart_of_accounts:
        if str(account.get("code", "")) == str(account_id) or str(account.get("id", "")) == str(account_id):
            return True
    logger.warning("account_not_found", account_id=account_id)
    return False


def _get_nested_value(data: Dict[str, Any], field: str) -> Any:
    if not field:
        return data
    parts = field.split(".")
    current = data
    for part in parts:
        if isinstance(current, dict):
            current = current.get(part)
        elif isinstance(current, list) and part.isdigit():
            idx = int(part)
            current = current[idx] if idx < len(current) else None
        else:
            return None
        if current is None:
            return None
    return current


def _evaluate_condition(value: Any, condition: str, rule: Dict[str, Any]) -> bool:
    if not condition:
        return True

    expected = rule.get("expected")
    min_val = rule.get("min")
    max_val = rule.get("max")
    pattern = rule.get("pattern")

    cond = condition.lower().strip()

    if cond == "required":
        return value is not None and value != "" and value != 0

    if cond == "not_empty":
        return bool(value)

    if cond == "equals":
        return value == expected

    if cond == "not_equals":
        return value != expected

    if cond == "greater_than":
        try:
            return float(value) > float(expected)
        except (TypeError, ValueError):
            return False

    if cond == "less_than":
        try:
            return float(value) < float(expected)
        except (TypeError, ValueError):
            return False

    if cond == "between":
        try:
            return float(min_val) <= float(value) <= float(max_val)
        except (TypeError, ValueError):
            return False

    if cond == "in_list":
        allowed = rule.get("allowed_values", [])
        return value in allowed

    if cond == "is_type":
        type_name = rule.get("expected_type", "str")
        type_map = {"str": str, "int": int, "float": (int, float), "bool": bool, "list": list, "dict": dict}
        expected_type = type_map.get(type_name, str)
        return isinstance(value, expected_type)

    if cond == "positive":
        try:
            return float(value) > 0
        except (TypeError, ValueError):
            return False

    if cond == "non_negative":
        try:
            return float(value) >= 0
        except (TypeError, ValueError):
            return False

    if cond == "matches_pattern":
        import re
        if pattern and isinstance(value, str):
            return bool(re.match(pattern, value))
        return False

    if cond == "custom":
        custom_fn = rule.get("validator")
        if callable(custom_fn):
            try:
                return bool(custom_fn(value, rule))
            except Exception:
                return False
        return True

    return True

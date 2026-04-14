# Author: Quadri Atharu
"""Base agent class for all Naija Finance Accounting Intelligence Engine agents."""

from __future__ import annotations

import time
import traceback
import uuid
from datetime import datetime
from typing import Any, Callable, Dict, List, Optional

from ..core.exceptions import EngineError
from ..core.logging import get_logger
from ..core.registry import ModuleRegistry

logger = get_logger(__name__)

_global_registry = ModuleRegistry()


def get_global_registry() -> ModuleRegistry:
    return _global_registry


class ActivityLogEntry:
    """Single activity log record."""

    __slots__ = ("id", "agent_name", "action", "input_summary", "output_summary", "duration_ms", "success", "timestamp")

    def __init__(
        self,
        agent_name: str,
        action: str,
        input_summary: str,
        output_summary: str,
        duration_ms: float,
        success: bool,
    ) -> None:
        self.id = str(uuid.uuid4())
        self.agent_name = agent_name
        self.action = action
        self.input_summary = input_summary
        self.output_summary = output_summary
        self.duration_ms = duration_ms
        self.success = success
        self.timestamp = datetime.now().isoformat()

    def to_dict(self) -> Dict[str, Any]:
        return {
            "id": self.id,
            "agent_name": self.agent_name,
            "action": self.action,
            "input_summary": self.input_summary,
            "output_summary": self.output_summary,
            "duration_ms": self.duration_ms,
            "success": self.success,
            "timestamp": self.timestamp,
        }


class ErrorResult:
    """Structured error result returned when a skill execution fails."""

    __slots__ = ("error_type", "message", "code", "details", "traceback", "timestamp")

    def __init__(
        self,
        error_type: str,
        message: str,
        code: str = "AGENT_ERROR",
        details: Optional[Dict[str, Any]] = None,
        tb: str = "",
    ) -> None:
        self.error_type = error_type
        self.message = message
        self.code = code
        self.details = details or {}
        self.traceback = tb
        self.timestamp = datetime.now().isoformat()

    def to_dict(self) -> Dict[str, Any]:
        logger.error("agent_error", error_type=self.error_type, message=self.message, code=self.code, traceback=self.traceback)
        return {
            "success": False,
            "error": {
                "type": self.error_type,
                "message": self.message,
                "code": self.code,
                "details": self.details,
                "timestamp": self.timestamp,
            },
        }


class BaseAgent:
    """Base agent class providing skill registration, execution, logging, and error handling.

    Subclasses register skills in __init__ via self.register_skill(name, handler).
    The execute() method dispatches to the registered handler, wraps timing, and logs
    the activity automatically.
    """

    agent_name: str = "base_agent"

    def __init__(self, registry: Optional[ModuleRegistry] = None) -> None:
        self._skills: Dict[str, Callable[..., Any]] = {}
        self._log: List[ActivityLogEntry] = []
        self._registry = registry or get_global_registry()
        self._register_with_registry()

    def _register_with_registry(self) -> None:
        self._registry.register(self.agent_name, self, metadata={"class": type(self).__name__, "skills": list(self._skills.keys())})

    def register_skill(self, name: str, handler: Callable[..., Any]) -> None:
        name_lower = name.lower().strip()
        if name_lower in self._skills:
            logger.warning("skill_overwrite", agent=self.agent_name, skill=name_lower)
        self._skills[name_lower] = handler
        logger.info("skill_registered", agent=self.agent_name, skill=name_lower)

    def list_skills(self) -> List[str]:
        return sorted(self._skills.keys())

    def execute(self, skill_name: str, data: Dict[str, Any]) -> Dict[str, Any]:
        start = time.perf_counter()
        skill_lower = skill_name.lower().strip()
        handler = self._skills.get(skill_lower)
        if handler is None:
            error_result = self.handle_error(
                EngineError(f"Skill '{skill_lower}' not found on agent '{self.agent_name}'", code="SKILL_NOT_FOUND")
            )
            self.log_activity(skill_lower, str(data)[:200], error_result.to_dict().__repr__()[:200], 0.0, False)
            return error_result.to_dict()

        input_summary = str(data)[:200]
        try:
            result = handler(data)
            duration_ms = round((time.perf_counter() - start) * 1000, 2)
            output_summary = str(result)[:200]
            self.log_activity(skill_lower, input_summary, output_summary, duration_ms, True)
            if isinstance(result, dict):
                result["_meta"] = {
                    "agent": self.agent_name,
                    "skill": skill_lower,
                    "duration_ms": duration_ms,
                    "success": True,
                }
                return result
            return {"result": result, "_meta": {"agent": self.agent_name, "skill": skill_lower, "duration_ms": duration_ms, "success": True}}
        except Exception as exc:
            duration_ms = round((time.perf_counter() - start) * 1000, 2)
            error_result = self.handle_error(exc)
            self.log_activity(skill_lower, input_summary, error_result.message[:200], duration_ms, False)
            return error_result.to_dict()

    def handle_error(self, error: Exception) -> ErrorResult:
        if isinstance(error, EngineError):
            return ErrorResult(
                error_type=type(error).__name__,
                message=error.message,
                code=error.code,
                details=error.details,
                tb=traceback.format_exc(),
            )
        return ErrorResult(
            error_type=type(error).__name__,
            message=str(error),
            code="UNHANDLED_ERROR",
            details={"exception_class": type(error).__name__},
            tb=traceback.format_exc(),
        )

    def log_activity(
        self,
        action: str,
        input_summary: str,
        output_summary: str,
        duration_ms: float,
        success: bool,
    ) -> None:
        entry = ActivityLogEntry(
            agent_name=self.agent_name,
            action=action,
            input_summary=input_summary,
            output_summary=output_summary,
            duration_ms=duration_ms,
            success=success,
        )
        self._log.append(entry)
        logger.info(
            "agent_activity",
            agent=self.agent_name,
            action=action,
            duration_ms=duration_ms,
            success=success,
        )

    def get_activity_log(self, limit: int = 100) -> List[Dict[str, Any]]:
        return [e.to_dict() for e in self._log[-limit:]]

    def health_check(self) -> bool:
        return True

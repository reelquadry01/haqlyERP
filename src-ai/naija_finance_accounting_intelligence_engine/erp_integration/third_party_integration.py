# Author: Quadri Atharu
"""Third-party integration registry for external service connections."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Callable, Dict, List, Optional

from ..core.exceptions import EngineError
from ..core.logging import get_logger

logger = get_logger(__name__)

IntegrationHandler = Callable[[str, Dict[str, Any]], Dict[str, Any]]


class IntegrationConfig:
    """Configuration for a third-party integration."""

    __slots__ = ("name", "base_url", "api_key", "secret", "extra", "created_at", "updated_at")

    def __init__(
        self,
        name: str,
        base_url: str = "",
        api_key: str = "",
        secret: str = "",
        extra: Optional[Dict[str, Any]] = None,
    ) -> None:
        self.name = name
        self.base_url = base_url
        self.api_key = api_key
        self.secret = secret
        self.extra = extra or {}
        self.created_at = datetime.now().isoformat()
        self.updated_at = datetime.now().isoformat()

    def to_dict(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "base_url": self.base_url,
            "api_key": _mask_secret(self.api_key),
            "extra": self.extra,
            "created_at": self.created_at,
            "updated_at": self.updated_at,
        }


class IntegrationRegistry:
    """Registry for managing third-party integrations with dynamic handler dispatch."""

    def __init__(self) -> None:
        self._integrations: Dict[str, IntegrationConfig] = {}
        self._handlers: Dict[str, IntegrationHandler] = {}
        self._call_log: List[Dict[str, Any]] = []

    def register_integration(
        self,
        name: str,
        config: Dict[str, Any],
        handler: Optional[IntegrationHandler] = None,
    ) -> Dict[str, Any]:
        name_lower = name.lower().strip()
        integration_config = IntegrationConfig(
            name=name_lower,
            base_url=config.get("base_url", ""),
            api_key=config.get("api_key", ""),
            secret=config.get("secret", ""),
            extra=config.get("extra", {}),
        )
        self._integrations[name_lower] = integration_config

        if handler:
            self._handlers[name_lower] = handler

        logger.info("integration_registered", name=name_lower)
        return {
            "success": True,
            "integration": name_lower,
            "config": integration_config.to_dict(),
            "has_handler": handler is not None,
        }

    def call_integration(
        self,
        name: str,
        action: str,
        data: Dict[str, Any],
    ) -> Dict[str, Any]:
        name_lower = name.lower().strip()
        config = self._integrations.get(name_lower)
        if config is None:
            raise EngineError(
                f"Integration '{name_lower}' is not registered",
                code="INTEGRATION_NOT_FOUND",
            )

        handler = self._handlers.get(name_lower)
        if handler is None:
            return self._default_call(config, action, data)

        call_start = datetime.now()
        try:
            result = handler(action, data)
            self._log_call(name_lower, action, True, result, call_start)
            return result
        except Exception as exc:
            error_result = {
                "success": False,
                "integration": name_lower,
                "action": action,
                "error": str(exc),
                "error_type": type(exc).__name__,
            }
            self._log_call(name_lower, action, False, error_result, call_start)
            raise EngineError(
                f"Integration '{name_lower}' call failed for action '{action}': {exc}",
                code="INTEGRATION_CALL_ERROR",
                details={"integration": name_lower, "action": action},
            )

    def list_integrations(self) -> List[Dict[str, Any]]:
        return [
            {
                **config.to_dict(),
                "has_handler": name in self._handlers,
            }
            for name, config in self._integrations.items()
        ]

    def unregister_integration(self, name: str) -> Dict[str, Any]:
        name_lower = name.lower().strip()
        if name_lower not in self._integrations:
            raise EngineError(
                f"Integration '{name_lower}' is not registered",
                code="INTEGRATION_NOT_FOUND",
            )
        del self._integrations[name_lower]
        self._handlers.pop(name_lower, None)
        logger.info("integration_unregistered", name=name_lower)
        return {"success": True, "unregistered": name_lower}

    def get_call_log(self, limit: int = 100) -> List[Dict[str, Any]]:
        return self._call_log[-limit:]

    def _default_call(
        self,
        config: IntegrationConfig,
        action: str,
        data: Dict[str, Any],
    ) -> Dict[str, Any]:
        if not config.base_url:
            return {
                "success": False,
                "integration": config.name,
                "action": action,
                "error": "No base_url configured and no custom handler registered",
            }

        try:
            import httpx
            url = f"{config.base_url.rstrip('/')}/{action.lstrip('/')}"
            headers: Dict[str, str] = {"Content-Type": "application/json"}
            if config.api_key:
                headers["Authorization"] = f"Bearer {config.api_key}"

            with httpx.Client(timeout=30.0) as client:
                response = client.post(url, json=data, headers=headers)

            result = {
                "success": response.status_code < 400,
                "integration": config.name,
                "action": action,
                "status_code": response.status_code,
                "response": response.json() if response.headers.get("content-type", "").startswith("application/json") else response.text,
            }
            self._log_call(config.name, action, result["success"], result, datetime.now())
            return result
        except ImportError:
            return {
                "success": False,
                "integration": config.name,
                "action": action,
                "error": "httpx not available for default integration calls",
            }
        except Exception as exc:
            error_result = {
                "success": False,
                "integration": config.name,
                "action": action,
                "error": str(exc),
            }
            self._log_call(config.name, action, False, error_result, datetime.now())
            return error_result

    def _log_call(
        self,
        integration: str,
        action: str,
        success: bool,
        result: Dict[str, Any],
        started_at: datetime,
    ) -> None:
        self._call_log.append({
            "integration": integration,
            "action": action,
            "success": success,
            "result_summary": str(result)[:500],
            "started_at": started_at.isoformat(),
            "logged_at": datetime.now().isoformat(),
        })


def _mask_secret(value: str) -> str:
    if len(value) <= 8:
        return "****" if value else ""
    return f"{value[:4]}****{value[-4:]}"

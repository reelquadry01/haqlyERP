# Author: Quadri Atharu
"""Module registry for dynamic registration of engines and agents."""

from __future__ import annotations

from typing import Any, Dict, List, Optional

from .exceptions import EngineError
from .logging import get_logger

logger = get_logger(__name__)


class ModuleRegistry:
    """Central registry for dynamically registering and looking up engine modules."""

    def __init__(self) -> None:
        self._modules: Dict[str, Any] = {}
        self._metadata: Dict[str, Dict[str, Any]] = {}

    def register(self, name: str, module: Any, metadata: Optional[Dict[str, Any]] = None) -> None:
        """Register a module by name with optional metadata."""
        name = name.lower().strip()
        if name in self._modules:
            logger.warning("module_overwrite", name=name)
        self._modules[name] = module
        self._metadata[name] = metadata or {
            "registered_at": __import__("datetime").datetime.now(__import__("datetime").timezone.utc).isoformat(),
            "class": type(module).__name__,
        }
        logger.info("module_registered", name=name, class_=type(module).__name__)

    def unregister(self, name: str) -> None:
        """Remove a module from the registry."""
        name = name.lower().strip()
        if name not in self._modules:
            raise EngineError(f"Module '{name}' is not registered", code="MODULE_NOT_FOUND")
        del self._modules[name]
        del self._metadata[name]
        logger.info("module_unregistered", name=name)

    def get(self, name: str) -> Any:
        """Retrieve a module by name."""
        name = name.lower().strip()
        if name not in self._modules:
            raise EngineError(f"Module '{name}' is not registered", code="MODULE_NOT_FOUND")
        return self._modules[name]

    def has(self, name: str) -> bool:
        """Check whether a module is registered."""
        return name.lower().strip() in self._modules

    def list_modules(self) -> List[str]:
        """Return a sorted list of all registered module names."""
        return sorted(self._modules.keys())

    def get_metadata(self, name: str) -> Dict[str, Any]:
        """Return metadata for a registered module."""
        name = name.lower().strip()
        if name not in self._metadata:
            raise EngineError(f"No metadata for module '{name}'", code="MODULE_NOT_FOUND")
        return self._metadata[name]

    def all_status(self) -> Dict[str, Dict[str, Any]]:
        """Return health status for every registered module."""
        status: Dict[str, Dict[str, Any]] = {}
        for name, module in self._modules.items():
            healthy = True
            if hasattr(module, "health_check"):
                try:
                    healthy = bool(module.health_check())
                except Exception:
                    healthy = False
            status[name] = {
                "registered": True,
                "healthy": healthy,
                "class": type(module).__name__,
                "metadata": self._metadata.get(name, {}),
            }
        return status

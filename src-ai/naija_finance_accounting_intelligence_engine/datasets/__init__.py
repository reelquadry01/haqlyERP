# Author: Quadri Atharu
"""Nigerian tax and economic datasets for the HAQLY AI Finance Engine."""

import json
from pathlib import Path
from typing import Any, Dict, Optional

_DATASETS_DIR = Path(__file__).parent


def load_dataset(*path_parts: str) -> Dict[str, Any]:
    """Load a JSON dataset from the datasets directory."""
    file_path = _DATASETS_DIR.joinpath(*path_parts)
    if not file_path.exists():
        raise FileNotFoundError(f"Dataset not found: {file_path}")
    with open(file_path, "r", encoding="utf-8") as f:
        return json.load(f)


def list_datasets() -> Dict[str, Any]:
    """List all available dataset files."""
    datasets: Dict[str, Any] = {}
    for json_file in _DATASETS_DIR.rglob("*.json"):
        relative = json_file.relative_to(_DATASETS_DIR)
        datasets[str(relative).replace("\\", "/")] = str(json_file)
    return {"datasets": datasets, "count": len(datasets)}


def get_vat_rules() -> Dict[str, Any]:
    """Load Nigerian VAT rules dataset."""
    return load_dataset("nigerian_tax", "vat_rules.json")


def get_wht_tables() -> Dict[str, Any]:
    """Load Nigerian WHT tables dataset."""
    return load_dataset("nigerian_tax", "wht_tables.json")


def get_cit_thresholds() -> Dict[str, Any]:
    """Load Nigerian CIT thresholds dataset."""
    return load_dataset("nigerian_tax", "cit_thresholds.json")


def get_inflation_history() -> Dict[str, Any]:
    """Load Nigerian inflation history dataset."""
    return load_dataset("nigerian_economic", "inflation_history.json")

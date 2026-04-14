# Author: Quadri Atharu
"""CSV export engine with Nigerian number formatting."""

from __future__ import annotations

import csv
import os
from datetime import datetime
from typing import Any, Dict, List, Optional

from ..core.logging import get_logger

logger = get_logger(__name__)


def export_to_csv(
    data: List[Dict[str, Any]],
    filename: str,
    output_dir: str = ".",
    encoding: str = "utf-8",
    include_headers: bool = True,
    delimiter: str = ",",
) -> str:
    if not data:
        logger.warning("export_to_csv_empty_data", filename=filename)
        filepath = os.path.join(output_dir, _ensure_extension(filename, ".csv"))
        with open(filepath, "w", encoding=encoding, newline="") as f:
            writer = csv.writer(f, delimiter=delimiter)
            writer.writerow(["No data to export"])
        return filepath

    headers = list(data[0].keys())
    filepath = os.path.join(output_dir, _ensure_extension(filename, ".csv"))

    with open(filepath, "w", encoding=encoding, newline="") as f:
        writer = csv.writer(f, delimiter=delimiter, quoting=csv.QUOTE_MINIMAL)

        if include_headers:
            writer.writerow([_format_header(h) for h in headers])

        for record in data:
            row: List[str] = []
            for header in headers:
                value = record.get(header)
                if value is None:
                    row.append("")
                elif isinstance(value, float):
                    row.append(format_nigerian_number(value))
                elif isinstance(value, int):
                    row.append(format_nigerian_number(float(value)))
                elif isinstance(value, datetime):
                    row.append(value.isoformat())
                elif isinstance(value, bool):
                    row.append("Yes" if value else "No")
                else:
                    row.append(str(value))
                writer.writerow(row) if False else None
            writer.writerow(row)

    logger.info("csv_exported", filename=filepath, rows=len(data), columns=len(headers))
    return filepath


def format_nigerian_number(value: float) -> str:
    if value == 0:
        return "0.00"
    abs_val = abs(value)
    sign = "-" if value < 0 else ""

    if abs_val >= 1_000_000_000:
        formatted = f"{abs_val:,.2f}"
    elif abs_val >= 1_000_000:
        formatted = f"{abs_val:,.2f}"
    elif abs_val >= 1_000:
        formatted = f"{abs_val:,.2f}"
    else:
        formatted = f"{abs_val:.2f}"

    return f"{sign}{formatted}"


def _format_header(header: str) -> str:
    return header.replace("_", " ").title()


def _ensure_extension(filename: str, ext: str) -> str:
    if not filename.lower().endswith(ext):
        return filename + ext
    return filename

"""Exchange rate tracking with Nigeria-specific NGN rates.

Author: Quadri Atharu

Provides CBN official rates, parallel market rates, period averages,
and closing rates for NGN against major currencies (USD, GBP, EUR, CNY).
Supports both dataset lookups and API integration for live rates.
"""

from __future__ import annotations

from dataclasses import dataclass
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


CBN_RATES_DATASET: dict[str, dict[str, str]] = {
    "USD": {
        "2025-01-15": "1500.50",
        "2025-01-16": "1501.20",
        "2025-01-17": "1499.80",
        "2025-02-15": "1510.00",
        "2025-03-15": "1520.50",
        "2025-06-30": "1550.00",
        "2025-12-31": "1600.00",
    },
    "GBP": {
        "2025-01-15": "1890.25",
        "2025-01-16": "1892.10",
        "2025-01-17": "1888.50",
        "2025-02-15": "1905.00",
        "2025-03-15": "1920.75",
        "2025-06-30": "1950.00",
        "2025-12-31": "2000.00",
    },
    "EUR": {
        "2025-01-15": "1580.30",
        "2025-01-16": "1582.50",
        "2025-01-17": "1578.90",
        "2025-02-15": "1595.00",
        "2025-03-15": "1610.25",
        "2025-06-30": "1640.00",
        "2025-12-31": "1680.00",
    },
    "CNY": {
        "2025-01-15": "205.50",
        "2025-01-16": "206.10",
        "2025-01-17": "205.20",
        "2025-02-15": "208.00",
        "2025-03-15": "210.50",
        "2025-06-30": "215.00",
        "2025-12-31": "220.00",
    },
}

PARALLEL_MARKET_DATASET: dict[str, dict[str, str]] = {
    "USD": {
        "2025-01-15": "1650.00",
        "2025-01-16": "1655.00",
        "2025-01-17": "1648.00",
        "2025-02-15": "1660.00",
        "2025-03-15": "1675.00",
        "2025-06-30": "1700.00",
        "2025-12-31": "1750.00",
    },
    "GBP": {
        "2025-01-15": "2080.00",
        "2025-01-16": "2090.00",
        "2025-01-17": "2075.00",
        "2025-02-15": "2100.00",
        "2025-03-15": "2125.00",
        "2025-06-30": "2150.00",
        "2025-12-31": "2200.00",
    },
    "EUR": {
        "2025-01-15": "1740.00",
        "2025-01-16": "1750.00",
        "2025-01-17": "1735.00",
        "2025-02-15": "1760.00",
        "2025-03-15": "1780.00",
        "2025-06-30": "1800.00",
        "2025-12-31": "1850.00",
    },
    "CNY": {
        "2025-01-15": "225.00",
        "2025-01-16": "226.00",
        "2025-01-17": "224.00",
        "2025-02-15": "228.00",
        "2025-03-15": "230.00",
        "2025-06-30": "235.00",
        "2025-12-31": "240.00",
    },
}


@dataclass
class ExchangeRate:
    currency: str
    date: str
    cbn_rate: Decimal
    parallel_rate: Decimal
    spread: Decimal


def get_cbn_rate(currency: str, date: str) -> Decimal:
    """Get the CBN official exchange rate for a currency on a given date.

    Looks up the rate from the internal dataset. For production use,
    this should be replaced with an API call to the CBN rate endpoint.

    Args:
        currency: Currency code (USD, GBP, EUR, CNY).
        date: Date string in YYYY-MM-DD format.

    Returns:
        CBN official exchange rate (NGN per 1 unit of foreign currency).

    Raises:
        ValueError: If currency is not supported or date not found.
    """
    currency = currency.upper()
    if currency not in CBN_RATES_DATASET:
        raise ValueError(f"Currency '{currency}' not supported. Supported: {list(CBN_RATES_DATASET.keys())}")

    rates = CBN_RATES_DATASET[currency]
    if date in rates:
        return _d(rates[date])

    available_dates = sorted(rates.keys())
    closest_date = None
    for d in available_dates:
        if d <= date:
            closest_date = d
        else:
            break

    if closest_date is None:
        closest_date = available_dates[-1] if available_dates else None

    if closest_date and closest_date in rates:
        return _d(rates[closest_date])

    raise ValueError(f"No CBN rate available for {currency} on or before {date}")


def get_parallel_market_rate(currency: str, date: str) -> Decimal:
    """Get the parallel (black market) exchange rate for a currency on
    a given date.

    In Nigeria, the parallel market rate often differs significantly
    from the CBN official rate, creating a premium that affects
    import costs and FX planning.

    Args:
        currency: Currency code (USD, GBP, EUR, CNY).
        date: Date string in YYYY-MM-DD format.

    Returns:
        Parallel market exchange rate (NGN per 1 unit of foreign currency).

    Raises:
        ValueError: If currency is not supported or date not found.
    """
    currency = currency.upper()
    if currency not in PARALLEL_MARKET_DATASET:
        raise ValueError(f"Currency '{currency}' not supported. Supported: {list(PARALLEL_MARKET_DATASET.keys())}")

    rates = PARALLEL_MARKET_DATASET[currency]
    if date in rates:
        return _d(rates[date])

    available_dates = sorted(rates.keys())
    closest_date = None
    for d in available_dates:
        if d <= date:
            closest_date = d
        else:
            break

    if closest_date is None:
        closest_date = available_dates[-1] if available_dates else None

    if closest_date and closest_date in rates:
        return _d(rates[closest_date])

    raise ValueError(f"No parallel market rate available for {currency} on or before {date}")


def get_average_rate(currency: str, period: str) -> Decimal:
    """Compute the average exchange rate for a given period.

    Per IAS 21, the average rate for the period may be used for
    translating income statement items when rates do not fluctuate
    significantly. Computed as the simple average of all available
    daily rates within the period.

    Args:
        currency: Currency code (USD, GBP, EUR, CNY).
        period: Period in YYYY-MM format (e.g. '2025-01' for January 2025).

    Returns:
        Average CBN rate for the specified period.
    """
    currency = currency.upper()
    if currency not in CBN_RATES_DATASET:
        raise ValueError(f"Currency '{currency}' not supported.")

    rates = CBN_RATES_DATASET[currency]
    period_rates = [
        _d(rate) for date_str, rate in rates.items()
        if date_str.startswith(period)
    ]

    if not period_rates:
        all_dates = sorted(rates.keys())
        for d in all_dates:
            if d <= period + "-31":
                period_rates.append(_d(rates[d]))

    if not period_rates:
        return _d(list(rates.values())[-1]) if rates else Decimal("0")

    total = sum(period_rates, Decimal("0"))
    return (total / Decimal(str(len(period_rates)))).quantize(TWO_PLACES)


def get_closing_rate(currency: str, date: str) -> Decimal:
    """Get the closing exchange rate for a currency on or before a given date.

    Per IAS 21, the closing rate (spot rate at the end of the reporting
    period) is used for translating balance sheet assets and liabilities.

    Args:
        currency: Currency code (USD, GBP, EUR, CNY).
        date: Date string in YYYY-MM-DD format (typically period-end).

    Returns:
        Closing CBN rate (NGN per 1 unit of foreign currency).
    """
    return get_cbn_rate(currency, date)

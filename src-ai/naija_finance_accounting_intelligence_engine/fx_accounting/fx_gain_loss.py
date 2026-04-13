"""FX gain/loss recognition per IAS 21.

Author: Quadri Atharu

Computes realized and unrealized foreign exchange gains and losses,
recognizes FX adjustments as journal entries, and applies the IAS 21
distinction between monetary and non-monetary items for exchange
difference treatment.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


MONETARY_ITEMS = {
    "cash", "bank", "receivables", "payables", "loans_receivable",
    "loans_payable", "accrued_income", "accrued_expense",
    "dividends_receivable", "dividends_payable",
    "interest_receivable", "interest_payable",
    "deposits", "borrowings",
}

NON_MONETARY_ITEMS = {
    "inventory", "ppe", "intangible_assets", "goodwill",
    "prepaid_expenses", "deferred_tax_assets", "investment_property",
    "biological_assets", "equity_investments_fvtoci",
    "right_of_use_asset", "share_capital",
}


@dataclass
class FXGainLoss:
    amount: Decimal
    currency: str
    gain_or_loss: str
    realized: bool
    description: str
    transaction_reference: str = ""


@dataclass
class FXJournalEntry:
    date: str
    debit_account: str
    credit_account: str
    amount: Decimal
    description: str
    fx_type: str = "unrealized"


def compute_realized_fx_gain_loss(
    transaction: dict[str, Any],
    settlement_rate: Decimal,
) -> FXGainLoss:
    """Compute realized FX gain or loss on settlement of a foreign
    currency monetary item.

    Per IAS 21.28, exchange differences arising on settlement of
    monetary items are recognized in profit or loss in the period
    of settlement.

    Args:
        transaction: Dict with 'original_amount', 'original_rate',
                    'currency', 'date', 'reference'.
        settlement_rate: The exchange rate at which the transaction
                        was actually settled.

    Returns:
        FXGainLoss with the realized gain/loss amount and direction.
    """
    original_amount = _d(transaction["original_amount"])
    original_rate = _d(transaction["original_rate"])
    settlement_rate = _d(settlement_rate)

    original_functional = (original_amount * original_rate).quantize(TWO_PLACES)
    settlement_functional = (original_amount * settlement_rate).quantize(TWO_PLACES)

    difference = settlement_functional - original_functional
    currency = transaction.get("currency", "USD")

    if difference > Decimal("0"):
        gain_loss = "loss"
        description = (
            f"Realized FX loss on settlement of {original_amount} {currency}: "
            f"original rate {original_rate}, settled at {settlement_rate}"
        )
    elif difference < Decimal("0"):
        gain_loss = "gain"
        description = (
            f"Realized FX gain on settlement of {original_amount} {currency}: "
            f"original rate {original_rate}, settled at {settlement_rate}"
        )
    else:
        gain_loss = "nil"
        description = f"No FX gain/loss on settlement of {original_amount} {currency}"

    return FXGainLoss(
        amount=abs(difference),
        currency=currency,
        gain_or_loss=gain_loss,
        realized=True,
        description=description,
        transaction_reference=transaction.get("reference", ""),
    )


def compute_unrealized_fx_gain_loss(
    positions: list[dict[str, Any]],
    closing_rate: Decimal,
) -> list[FXGainLoss]:
    """Compute unrealized FX gains and losses on outstanding monetary
    items at period-end.

    Per IAS 21.23, monetary items denominated in foreign currency are
    retranslated using the closing rate. Non-monetary items carried at
    historical cost are NOT retranslated.

    Args:
        positions: List of dicts with 'amount' (foreign currency),
                  'original_rate', 'currency', 'item_type' (monetary/
                  non-monetary), 'account', 'reference'.
        closing_rate: Closing rate for retranslation.

    Returns:
        List of FXGainLoss for each monetary item with unrealized difference.
    """
    results: list[FXGainLoss] = []
    closing_rate = _d(closing_rate)

    for pos in positions:
        item_type = pos.get("item_type", "monetary").lower()
        if item_type != "monetary":
            continue

        amount = _d(pos["amount"])
        original_rate = _d(pos["original_rate"])
        currency = pos.get("currency", "USD")

        original_functional = (amount * original_rate).quantize(TWO_PLACES)
        closing_functional = (amount * closing_rate).quantize(TWO_PLACES)
        difference = closing_functional - original_functional

        if difference > Decimal("0"):
            if amount > Decimal("0"):
                gain_loss = "loss"
            else:
                gain_loss = "gain"
        elif difference < Decimal("0"):
            if amount > Decimal("0"):
                gain_loss = "gain"
            else:
                gain_loss = "loss"
        else:
            gain_loss = "nil"

        results.append(FXGainLoss(
            amount=abs(difference),
            currency=currency,
            gain_or_loss=gain_loss,
            realized=False,
            description=(
                f"Unrealized FX {gain_loss} on {pos.get('account', 'monetary item')} "
                f"({amount} {currency}) at closing rate {closing_rate}"
            ),
            transaction_reference=pos.get("reference", ""),
        ))

    return results


def recognize_fx_adjustment(
    gain_loss: FXGainLoss,
    account_mapping: dict[str, str],
) -> list[FXJournalEntry]:
    """Generate journal entries to recognize an FX gain or loss adjustment.

    For realized gains/losses: recognized in P&L.
    For unrealized gains/losses on monetary items: recognized in P&L per IAS 21.
    For unrealized gains/losses on non-monetary items at FVTOCI: recognized in OCI.

    Args:
        gain_loss: The FXGainLoss to recognize.
        account_mapping: Dict mapping 'fx_gain_account', 'fx_loss_account',
                        'fx_oci_account', 'monetary_account' to account codes.

    Returns:
        List of FXJournalEntry objects for the recognition.
    """
    entries: list[FXJournalEntry] = []

    fx_gain_account = account_mapping.get("fx_gain_account", "FX Gain")
    fx_loss_account = account_mapping.get("fx_loss_account", "FX Loss")
    fx_oci_account = account_mapping.get("fx_oci_account", "FX Translation Reserve")
    monetary_account = account_mapping.get("monetary_account", "Monetary Item")

    if gain_loss.amount == Decimal("0"):
        return entries

    is_oci = not gain_loss.realized and gain_loss.description.lower().find("fvoci") >= 0

    if gain_loss.gain_or_loss == "gain":
        credit_account = fx_gain_account if not is_oci else fx_oci_account
        entries.append(FXJournalEntry(
            date="",
            debit_account=monetary_account,
            credit_account=credit_account,
            amount=gain_loss.amount,
            description=gain_loss.description,
            fx_type="unrealized" if not gain_loss.realized else "realized",
        ))
    elif gain_loss.gain_or_loss == "loss":
        debit_account = fx_loss_account if not is_oci else fx_oci_account
        entries.append(FXJournalEntry(
            date="",
            debit_account=debit_account,
            credit_account=monetary_account,
            amount=gain_loss.amount,
            description=gain_loss.description,
            fx_type="unrealized" if not gain_loss.realized else "realized",
        ))

    return entries

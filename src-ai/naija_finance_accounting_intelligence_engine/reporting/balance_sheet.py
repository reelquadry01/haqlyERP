# Author: Quadri Atharu
"""Balance Sheet generation engine with classified format."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)


class BalanceSheetEngine:
    """Balance Sheet generation engine with classified format."""

    def generate(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate a classified Balance Sheet (Statement of Financial Position)."""
        company_id = data.get("company_id", "")
        as_of = data.get("as_of", "")
        currency = data.get("currency", "NGN")

        ppe = float(data.get("property_plant_equipment", 0))
        intangible_assets = float(data.get("intangible_assets", 0))
        investment_in_associates = float(data.get("investment_in_associates", 0))
        other_non_current = float(data.get("other_non_current_assets", 0))
        total_non_current_assets = round(ppe + intangible_assets + investment_in_associates + other_non_current, 2)

        inventory = float(data.get("inventory", 0))
        trade_receivables = float(data.get("trade_receivables", 0))
        other_receivables = float(data.get("other_receivables", 0))
        cash = float(data.get("cash", 0))
        short_term_investments = float(data.get("short_term_investments", 0))
        other_current_assets = float(data.get("other_current_assets", 0))
        total_current_assets = round(inventory + trade_receivables + other_receivables + cash + short_term_investments + other_current_assets, 2)

        total_assets = round(total_non_current_assets + total_current_assets, 2)

        share_capital = float(data.get("share_capital", 0))
        share_premium = float(data.get("share_premium", 0))
        retained_earnings = float(data.get("retained_earnings", 0))
        revaluation_reserve = float(data.get("revaluation_reserve", 0))
        other_reserves = float(data.get("other_reserves", 0))
        total_equity = round(share_capital + share_premium + retained_earnings + revaluation_reserve + other_reserves, 2)

        long_term_loans = float(data.get("long_term_loans", 0))
        deferred_tax = float(data.get("deferred_tax_liability", 0))
        other_non_current_liab = float(data.get("other_non_current_liabilities", 0))
        total_non_current_liabilities = round(long_term_loans + deferred_tax + other_non_current_liab, 2)

        trade_payables = float(data.get("trade_payables", 0))
        short_term_loans = float(data.get("short_term_loans", 0))
        tax_payable = float(data.get("tax_payable", 0))
        other_current_liab = float(data.get("other_current_liabilities", 0))
        total_current_liabilities = round(trade_payables + short_term_loans + tax_payable + other_current_liab, 2)

        total_liabilities = round(total_non_current_liabilities + total_current_liabilities, 2)
        balance_check = abs(total_assets - (total_liabilities + total_equity)) < 0.01

        return {
            "report_type": "balance_sheet",
            "statement_name": "Statement of Financial Position",
            "company_id": company_id,
            "as_of": as_of,
            "currency": currency,
            "non_current_assets": {
                "property_plant_equipment": ppe,
                "intangible_assets": intangible_assets,
                "investment_in_associates": investment_in_associates,
                "other_non_current_assets": other_non_current,
                "total_non_current_assets": total_non_current_assets,
            },
            "current_assets": {
                "inventory": inventory,
                "trade_receivables": trade_receivables,
                "other_receivables": other_receivables,
                "cash_and_cash_equivalents": cash,
                "short_term_investments": short_term_investments,
                "other_current_assets": other_current_assets,
                "total_current_assets": total_current_assets,
            },
            "total_assets": total_assets,
            "equity": {
                "share_capital": share_capital,
                "share_premium": share_premium,
                "retained_earnings": retained_earnings,
                "revaluation_reserve": revaluation_reserve,
                "other_reserves": other_reserves,
                "total_equity": total_equity,
            },
            "non_current_liabilities": {
                "long_term_loans": long_term_loans,
                "deferred_tax": deferred_tax,
                "other_non_current": other_non_current_liab,
                "total_non_current_liabilities": total_non_current_liabilities,
            },
            "current_liabilities": {
                "trade_payables": trade_payables,
                "short_term_loans": short_term_loans,
                "tax_payable": tax_payable,
                "other_current_liabilities": other_current_liab,
                "total_current_liabilities": total_current_liabilities,
            },
            "total_liabilities": total_liabilities,
            "balance_check": balance_check,
            "balance_check_detail": f"Assets ({total_assets}) = Liabilities ({total_liabilities}) + Equity ({total_equity}) = {total_liabilities + total_equity}",
            "generated_at": datetime.now().isoformat(),
        }

    def health_check(self) -> bool:
        return True

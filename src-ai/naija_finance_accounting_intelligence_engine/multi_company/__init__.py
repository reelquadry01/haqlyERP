"""Multi-company consolidation and intercompany elimination module.

Author: Quadri Atharu

Provides consolidated financial statement generation, intercompany
transaction elimination, minority interest computation, and group
reporting structure management for Nigerian multi-entity groups.
"""

from .consolidation import (
    generate_consolidated_balance_sheet,
    generate_consolidated_income_statement,
    adjust_for_investments,
    eliminate_intercompany,
)
from .intercompany import (
    identify_intercompany_transactions,
    eliminate_ic_sales_revenue,
    eliminate_ic_receivables_payables,
    eliminate_ic_inventory_profit,
    eliminate_ic_loans_interest,
)
from .minority_interest import (
    compute_minority_interest,
    compute_nci_on_balance_sheet,
    allocate_income,
)
from .group_reporting import (
    define_group_structure,
    get_consolidation_scope,
    check_control_threshold,
)

__all__ = [
    "generate_consolidated_balance_sheet",
    "generate_consolidated_income_statement",
    "adjust_for_investments",
    "eliminate_intercompany",
    "identify_intercompany_transactions",
    "eliminate_ic_sales_revenue",
    "eliminate_ic_receivables_payables",
    "eliminate_ic_inventory_profit",
    "eliminate_ic_loans_interest",
    "compute_minority_interest",
    "compute_nci_on_balance_sheet",
    "allocate_income",
    "define_group_structure",
    "get_consolidation_scope",
    "check_control_threshold",
]

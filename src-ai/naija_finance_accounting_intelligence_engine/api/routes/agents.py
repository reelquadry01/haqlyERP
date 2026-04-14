# Author: Quadri Atharu
"""Agent routes — execute skills, query status, and view activity logs."""

from __future__ import annotations

from typing import Any, Dict, List, Optional

from fastapi import APIRouter, HTTPException, status

from ...agents.base_agent import BaseAgent, get_global_registry

router = APIRouter(prefix="/agents", tags=["Agents"])

_registry = get_global_registry()

AGENT_TYPES: Dict[str, type] = {
    "accounting": BaseAgent,
    "tax": BaseAgent,
    "reporting": BaseAgent,
    "treasury": BaseAgent,
    "audit": BaseAgent,
    "analysis": BaseAgent,
    "erp": BaseAgent,
}

_agent_instances: Dict[str, BaseAgent] = {}


def _get_or_create_agent(agent_type: str) -> BaseAgent:
    """Get an existing agent instance or create a new one."""
    normalized = agent_type.lower().strip()
    if normalized in _agent_instances:
        return _agent_instances[normalized]

    agent_cls = AGENT_TYPES.get(normalized, BaseAgent)
    agent = agent_cls(registry=_registry)
    agent.agent_name = normalized

    _register_default_skills(agent, normalized)
    _agent_instances[normalized] = agent
    return agent


def _register_default_skills(agent: BaseAgent, agent_type: str) -> None:
    """Register default skills for each agent type."""
    if agent_type == "accounting":
        from ...accounting.methods import AccountingMethods
        from ...accounting.journal_engine import JournalEngine
        methods = AccountingMethods()
        journal = JournalEngine()
        agent.register_skill("process_accrual", lambda data: methods.process_accrual(data))
        agent.register_skill("create_journal", lambda data: journal.create_journal_entry(data))
    elif agent_type == "tax":
        from ...tax_engine.vat import VatEngine
        from ...tax_engine.wht import WhtEngine
        from ...tax_engine.cit import CitEngine
        vat = VatEngine()
        wht = WhtEngine()
        cit = CitEngine()
        agent.register_skill("compute_vat", lambda data: vat.compute_output_vat(**data))
        agent.register_skill("compute_wht", lambda data: wht.compute_wht(**data))
        agent.register_skill("compute_cit", lambda data: cit.compute_cit(**data))
    elif agent_type == "reporting":
        from ...reporting.income_statement import IncomeStatementEngine
        from ...reporting.balance_sheet import BalanceSheetEngine
        income = IncomeStatementEngine()
        balance = BalanceSheetEngine()
        agent.register_skill("income_statement", lambda data: income.generate(data))
        agent.register_skill("balance_sheet", lambda data: balance.generate(data))
    elif agent_type == "treasury":
        from ...treasury.cash_position import CashPositionEngine
        from ...treasury.bank_reconciliation import BankReconciliationEngine
        cash = CashPositionEngine()
        bank_rec = BankReconciliationEngine()
        agent.register_skill("cash_position", lambda data: cash.compute_cash_position(data))
        agent.register_skill("bank_reconciliation", lambda data: bank_rec.reconcile(data))
    elif agent_type == "audit":
        from ...audit_intelligence.audit_trail_generation import AuditTrailEngine
        from ...audit_intelligence.exception_detection import ExceptionDetectionEngine
        trail = AuditTrailEngine()
        exceptions = ExceptionDetectionEngine()
        agent.register_skill("audit_trail", lambda data: trail.generate_trail_report(**data))
        agent.register_skill("detect_exceptions", lambda data: exceptions.detect_unusual_amounts(**data))
    elif agent_type == "analysis":
        from ...financial_analysis.profitability_ratios import ProfitabilityRatiosEngine
        from ...financial_analysis.trend_analysis import TrendAnalysisEngine
        profitability = ProfitabilityRatiosEngine()
        trend = TrendAnalysisEngine()
        agent.register_skill("ratios", lambda data: profitability.compute_all(data))
        agent.register_skill("trend", lambda data: trend.analyze_trend(data))
    elif agent_type == "erp":
        agent.register_skill("health_check", lambda data: {"healthy": True})


@router.post("/execute/{agent_type}")
async def execute_agent_skill(agent_type: str, body: Dict[str, Any]) -> Dict[str, Any]:
    """Execute a skill on a specific agent type.

    The request body must contain 'skill' (the skill name) and 'data' (the input data).
    """
    skill = body.get("skill")
    data = body.get("data", {})
    if not skill:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="skill is required in request body")
    try:
        agent = _get_or_create_agent(agent_type)
        result = agent.execute(skill, data)
        return result
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))


@router.get("/status")
async def get_agent_status() -> Dict[str, Any]:
    """Return all agent types and their registered skills."""
    agents_info: List[Dict[str, Any]] = []
    for agent_type in AGENT_TYPES:
        agent = _get_or_create_agent(agent_type)
        agents_info.append({
            "agent_type": agent_type,
            "skills": agent.list_skills(),
            "skill_count": len(agent.list_skills()),
        })
    return {"status": "success", "agents": agents_info, "total_agent_types": len(AGENT_TYPES)}


@router.get("/logs")
async def get_agent_logs(limit: int = 50) -> Dict[str, Any]:
    """Return recent agent activity logs across all agent types."""
    all_logs: List[Dict[str, Any]] = []
    for agent_type, agent in _agent_instances.items():
        logs = agent.get_activity_log(limit=limit)
        all_logs.extend(logs)
    all_logs.sort(key=lambda x: x.get("timestamp", ""), reverse=True)
    return {"status": "success", "logs": all_logs[:limit], "total": len(all_logs)}

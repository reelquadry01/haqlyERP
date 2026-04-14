# Author: Quadri Atharu
"""Audit Agent — generates audit trails, samples transactions, detects exceptions, and generates working papers."""

from __future__ import annotations

from typing import Any, Dict, List

from ..audit_intelligence import (
    AuditTrailEngine,
    AuditWorkingPapersEngine,
    ExceptionDetectionEngine,
    SamplingLogicEngine,
)
from .base_agent import BaseAgent


class AuditAgent(BaseAgent):
    """Agent responsible for audit intelligence operations."""

    agent_name = "audit_agent"

    def __init__(self) -> None:
        super().__init__()
        self._trail = AuditTrailEngine()
        self._sampling = SamplingLogicEngine()
        self._exceptions = ExceptionDetectionEngine()
        self._working_papers = AuditWorkingPapersEngine()
        self.register_skill("generate_audit_trail", self._generate_audit_trail)
        self.register_skill("sample_transactions", self._sample_transactions)
        self.register_skill("detect_exceptions", self._detect_exceptions)
        self.register_skill("generate_working_papers", self._generate_working_papers)

    def _generate_audit_trail(self, data: Dict[str, Any]) -> Dict[str, Any]:
        action = data.get("action", "generate_trail")
        if action == "record_event":
            event = data.get("event", {})
            entry = self._trail.record_event(event)
            return {"success": True, "action": "record_event", "trail_entry": entry}
        elif action == "query_trail":
            query = data.get("query", {})
            results = self._trail.query(query) if hasattr(self._trail, "query") else []
            return {"success": True, "action": "query_trail", "results": results}
        elif action == "get_full_trail":
            company_id = data.get("company_id")
            entity_type = data.get("entity_type")
            entity_id = data.get("entity_id")
            start_date = data.get("start_date")
            end_date = data.get("end_date")
            entries = self._trail.get_events(
                company_id=company_id,
                entity_type=entity_type,
                entity_id=entity_id,
                start_date=start_date,
                end_date=end_date,
            ) if hasattr(self._trail, "get_events") else []
            return {
                "success": True,
                "action": "get_full_trail",
                "entries": entries,
                "total_entries": len(entries) if isinstance(entries, list) else 0,
            }
        else:
            event = {
                "company_id": data.get("company_id", ""),
                "user_id": data.get("user_id"),
                "action": data.get("event_action", data.get("action", "")),
                "entity_type": data.get("entity_type", ""),
                "entity_id": data.get("entity_id", ""),
                "old_value": data.get("old_value"),
                "new_value": data.get("new_value"),
                "notes": data.get("notes"),
                "ip_address": data.get("ip_address"),
            }
            entry = self._trail.record_event(event)
            return {"success": True, "trail_entry": entry}

    def _sample_transactions(self, data: Dict[str, Any]) -> Dict[str, Any]:
        method = data.get("method", "random").lower()
        population = data.get("population", [])
        sample_size = int(data.get("sample_size", min(25, len(population) if population else 25)))

        sample_data = {
            "population": population,
            "sample_size": sample_size,
        }

        if method == "random":
            result = self._sampling.random_sample(sample_data)
        elif method == "systematic":
            sample_data["random_start"] = int(data.get("random_start", 0))
            result = self._sampling.systematic_sample(sample_data)
        elif method == "stratified":
            sample_data["strata_key"] = data.get("strata_key", "amount")
            sample_data["strata_definitions"] = data.get("strata_definitions", [])
            result = self._sampling.stratified_sample(sample_data) if hasattr(self._sampling, "stratified_sample") else self._sampling.random_sample(sample_data)
        elif method == "monetary_unit":
            sample_data["tolerable_error"] = float(data.get("tolerable_error", 0.05))
            sample_data["expected_error"] = float(data.get("expected_error", 0.01))
            sample_data["confidence_level"] = float(data.get("confidence_level", 0.95))
            result = self._sampling.monetary_unit_sample(sample_data) if hasattr(self._sampling, "monetary_unit_sample") else self._sampling.random_sample(sample_data)
        else:
            result = self._sampling.random_sample(sample_data)

        return {
            "success": True,
            "method": method,
            "sampling_result": result,
            "population_size": len(population),
            "requested_sample_size": sample_size,
        }

    def _detect_exceptions(self, data: Dict[str, Any]) -> Dict[str, Any]:
        detection_method = data.get("method", "benford").lower()
        results: Dict[str, Any] = {}

        if detection_method in ("benford", "all"):
            amounts = data.get("amounts", [])
            results["benford"] = self._exceptions.benford_analysis(amounts)

        if detection_method in ("duplicates", "all"):
            transactions = data.get("transactions", [])
            results["duplicates"] = self._exceptions.detect_duplicates(transactions) if hasattr(self._exceptions, "detect_duplicates") else {"message": "duplicate detection not available"}

        if detection_method in ("outliers", "all"):
            amounts = data.get("amounts", [])
            threshold = float(data.get("outlier_threshold", 3.0))
            results["outliers"] = self._exceptions.detect_outliers(amounts, threshold) if hasattr(self._exceptions, "detect_outliers") else {"message": "outlier detection not available"}

        if detection_method == "all":
            amounts = data.get("amounts", [])
            if "benford" not in results:
                results["benford"] = self._exceptions.benford_analysis(amounts)

        return {
            "success": True,
            "method": detection_method,
            "results": results,
        }

    def _generate_working_papers(self, data: Dict[str, Any]) -> Dict[str, Any]:
        working_paper = self._working_papers.generate_working_paper(data)
        return {
            "success": True,
            "working_paper": working_paper,
        }

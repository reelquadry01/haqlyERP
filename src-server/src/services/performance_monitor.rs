// Author: Quadri Atharu

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

const MAX_METRICS: usize = 10000;
const SLOW_QUERY_THRESHOLD_MS: u64 = 500;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    pub endpoint: String,
    pub method: String,
    pub duration_ms: u64,
    pub status_code: u16,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowQueryLog {
    pub query: String,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointStats {
    pub endpoint: String,
    pub request_count: u64,
    pub avg_duration_ms: f64,
    pub error_count: u64,
    pub error_rate: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Ok,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub status: HealthStatus,
    pub p95_threshold_ms: u64,
    pub critical_threshold_ms: u64,
    pub endpoints: Vec<EndpointHealth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointHealth {
    pub endpoint: String,
    pub p95_ms: f64,
    pub status: HealthStatus,
}

struct Inner {
    request_metrics: VecDeque<RequestMetrics>,
    slow_queries: VecDeque<SlowQueryLog>,
}

pub struct PerformanceMonitor {
    inner: Arc<RwLock<Inner>>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        PerformanceMonitor {
            inner: Arc::new(RwLock::new(Inner {
                request_metrics: VecDeque::with_capacity(MAX_METRICS),
                slow_queries: VecDeque::with_capacity(MAX_METRICS),
            })),
        }
    }

    pub async fn record_request(&self, endpoint: String, method: String, duration_ms: u64, status_code: u16) {
        let metric = RequestMetrics {
            endpoint,
            method,
            duration_ms,
            status_code,
            timestamp: Utc::now(),
        };
        let mut inner = self.inner.write().await;
        if inner.request_metrics.len() >= MAX_METRICS {
            inner.request_metrics.pop_front();
        }
        inner.request_metrics.push_back(metric);
    }

    pub async fn record_slow_query(&self, query: String, duration_ms: u64, endpoint: String) {
        if duration_ms < SLOW_QUERY_THRESHOLD_MS {
            return;
        }
        let log = SlowQueryLog {
            query,
            duration_ms,
            timestamp: Utc::now(),
            endpoint,
        };
        let mut inner = self.inner.write().await;
        if inner.slow_queries.len() >= MAX_METRICS {
            inner.slow_queries.pop_front();
        }
        inner.slow_queries.push_back(log);
    }

    pub async fn get_percentiles(&self, endpoint: &str) -> (f64, f64, f64) {
        let inner = self.inner.read().await;
        let cutoff = Utc::now() - chrono::Duration::hours(1);
        let mut durations: Vec<u64> = inner
            .request_metrics
            .iter()
            .filter(|m| m.endpoint == endpoint && m.timestamp >= cutoff)
            .map(|m| m.duration_ms)
            .collect();

        if durations.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        durations.sort_unstable();
        let p50 = percentile(&durations, 50);
        let p95 = percentile(&durations, 95);
        let p99 = percentile(&durations, 99);
        (p50, p95, p99)
    }

    pub async fn get_slow_queries(&self, limit: usize) -> Vec<SlowQueryLog> {
        let inner = self.inner.read().await;
        inner
            .slow_queries
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    pub async fn get_endpoint_stats(&self) -> Vec<EndpointStats> {
        let inner = self.inner.read().await;
        let cutoff = Utc::now() - chrono::Duration::hours(1);
        let mut grouped: HashMap<String, Vec<&RequestMetrics>> = HashMap::new();

        for m in inner.request_metrics.iter() {
            if m.timestamp >= cutoff {
                grouped.entry(m.endpoint.clone()).or_default().push(m);
            }
        }

        let mut stats = Vec::new();
        for (endpoint, metrics) in grouped {
            let request_count = metrics.len() as u64;
            let total_duration: u64 = metrics.iter().map(|m| m.duration_ms).sum();
            let avg_duration_ms = if request_count > 0 {
                total_duration as f64 / request_count as f64
            } else {
                0.0
            };
            let error_count = metrics.iter().filter(|m| m.status_code >= 400).count() as u64;
            let error_rate = if request_count > 0 {
                error_count as f64 / request_count as f64
            } else {
                0.0
            };

            let mut durations: Vec<u64> = metrics.iter().map(|m| m.duration_ms).collect();
            durations.sort_unstable();

            let p50_ms = percentile(&durations, 50);
            let p95_ms = percentile(&durations, 95);
            let p99_ms = percentile(&durations, 99);

            stats.push(EndpointStats {
                endpoint,
                request_count,
                avg_duration_ms,
                error_count,
                error_rate,
                p50_ms,
                p95_ms,
                p99_ms,
            });
        }

        stats.sort_by(|a, b| b.request_count.cmp(&a.request_count));
        stats
    }

    pub async fn get_health_status(&self) -> HealthReport {
        let p95_warning_ms: u64 = 1000;
        let p95_critical_ms: u64 = 5000;

        let endpoint_stats = self.get_endpoint_stats().await;
        let mut endpoint_healths = Vec::new();
        let mut overall = HealthStatus::Ok;

        for stat in &endpoint_stats {
            let status = if stat.p95_ms >= p95_critical_ms as f64 {
                HealthStatus::Critical
            } else if stat.p95_ms >= p95_warning_ms as f64 {
                HealthStatus::Warning
            } else {
                HealthStatus::Ok
            };

            if status == HealthStatus::Critical || (status == HealthStatus::Warning && overall != HealthStatus::Critical) {
                overall = status.clone();
            }

            endpoint_healths.push(EndpointHealth {
                endpoint: stat.endpoint.clone(),
                p95_ms: stat.p95_ms,
                status,
            });
        }

        HealthReport {
            status: overall,
            p95_threshold_ms: p95_warning_ms,
            critical_threshold_ms: p95_critical_ms,
            endpoints: endpoint_healths,
        }
    }
}

fn percentile(sorted: &[u64], pct: u8) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((pct as f64 / 100.0) * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)] as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_and_retrieve() {
        let monitor = PerformanceMonitor::new();
        monitor.record_request("/api/journals".into(), "GET".into(), 120, 200).await;
        monitor.record_request("/api/journals".into(), "GET".into(), 250, 200).await;
        monitor.record_request("/api/journals".into(), "POST".into(), 800, 201).await;

        let stats = monitor.get_endpoint_stats().await;
        assert!(!stats.is_empty());
    }

    #[tokio::test]
    async fn test_slow_query_threshold() {
        let monitor = PerformanceMonitor::new();
        monitor.record_slow_query("SELECT * FROM chart_of_accounts".into(), 300, "/api/accounts".into()).await;
        monitor.record_slow_query("SELECT * FROM journals".into(), 600, "/api/journals".into()).await;

        let slow = monitor.get_slow_queries(10).await;
        assert_eq!(slow.len(), 1);
        assert_eq!(slow[0].duration_ms, 600);
    }

    #[test]
    fn test_percentile_calculation() {
        let data = vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
        assert_eq!(percentile(&data, 50), 60.0);
        assert_eq!(percentile(&data, 95), 100.0);
        assert_eq!(percentile(&data, 99), 100.0);
    }

    #[tokio::test]
    async fn test_health_status() {
        let monitor = PerformanceMonitor::new();
        for _ in 0..5 {
            monitor.record_request("/api/health".into(), "GET".into(), 50, 200).await;
        }
        let report = monitor.get_health_status().await;
        assert_eq!(report.status, HealthStatus::Ok);
    }

    #[tokio::test]
    async fn test_circular_buffer_eviction() {
        let monitor = PerformanceMonitor::new();
        for i in 0..(MAX_METRICS + 100) {
            monitor.record_request("/api/test".into(), "GET".into(), (i % 100) as u64, 200).await;
        }
        let inner = monitor.inner.read().await;
        assert_eq!(inner.request_metrics.len(), MAX_METRICS);
    }
}

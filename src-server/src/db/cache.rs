// HAQLY ERP - In-Memory LRU Cache Layer
// Author: Quadri Atharu

use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

fn non_zero_cap(capacity: usize, fallback: usize) -> NonZeroUsize {
    NonZeroUsize::new(capacity)
        .or_else(|| NonZeroUsize::new(fallback))
        .unwrap_or(NonZeroUsize::MIN)
}

#[derive(Clone)]
struct CachedEntry<T: Clone> {
    data: T,
    cached_at: Instant,
    ttl: Duration,
}

impl<T: Clone> CachedEntry<T> {
    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }
}

pub struct AccountCache {
    cache: Arc<RwLock<LruCache<Uuid, CachedEntry<serde_json::Value>>>>,
    ttl: Duration,
}

impl AccountCache {
    pub fn new(capacity: usize, ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(non_zero_cap(capacity, 1000)))),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub async fn get(&self, key: Uuid) -> Option<serde_json::Value> {
        let cache = self.cache.read().await;
        if let Some(entry) = cache.peek(&key) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            }
        }
        None
    }

    pub async fn set(&self, key: Uuid, value: serde_json::Value) {
        let mut cache = self.cache.write().await;
        cache.put(key, CachedEntry {
            data: value,
            cached_at: Instant::now(),
            ttl: self.ttl,
        });
    }

    pub async fn invalidate(&self, key: Uuid) {
        let mut cache = self.cache.write().await;
        cache.pop(&key);
    }

    pub async fn invalidate_all(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}

pub struct PostingRuleCache {
    cache: Arc<RwLock<LruCache<String, CachedEntry<serde_json::Value>>>>,
    ttl: Duration,
}

impl PostingRuleCache {
    pub fn new(capacity: usize, ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(non_zero_cap(capacity, 500)))),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub async fn get(&self, key: &str) -> Option<serde_json::Value> {
        let cache = self.cache.read().await;
        if let Some(entry) = cache.peek(&key.to_string()) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            }
        }
        None
    }

    pub async fn set(&self, key: &str, value: serde_json::Value) {
        let mut cache = self.cache.write().await;
        cache.put(key.to_string(), CachedEntry {
            data: value,
            cached_at: Instant::now(),
            ttl: self.ttl,
        });
    }

    pub async fn invalidate_all(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}

pub struct TaxConfigCache {
    cache: Arc<RwLock<LruCache<String, CachedEntry<serde_json::Value>>>>,
    ttl: Duration,
}

impl TaxConfigCache {
    pub fn new(capacity: usize, ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(non_zero_cap(capacity, 200)))),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub async fn get(&self, company_id: &str, tax_type: &str) -> Option<serde_json::Value> {
        let key = format!("{}:{}", company_id, tax_type);
        let cache = self.cache.read().await;
        if let Some(entry) = cache.peek(&key) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            }
        }
        None
    }

    pub async fn set(&self, company_id: &str, tax_type: &str, value: serde_json::Value) {
        let key = format!("{}:{}", company_id, tax_type);
        let mut cache = self.cache.write().await;
        cache.put(key, CachedEntry {
            data: value,
            cached_at: Instant::now(),
            ttl: self.ttl,
        });
    }

    pub async fn invalidate_company(&self, company_id: &str) {
        let mut cache = self.cache.write().await;
        let keys_to_remove: Vec<String> = cache.iter()
            .filter(|(k, _)| k.starts_with(&format!("{}:", company_id)))
            .map(|(k, _)| k.clone())
            .collect();
        for key in keys_to_remove {
            cache.pop(&key);
        }
    }
}

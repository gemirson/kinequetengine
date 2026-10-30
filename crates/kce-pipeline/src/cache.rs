//! Multi-level cache for pipeline responses (FT-024).
//!
//! Three levels of caching with decreasing specificity:
//! - L1: exact request hash → response (O(1) lookup, FIFO eviction)
//! - L2: embedding similarity → nearest cached results (cosine similarity)
//! - L3: top-k result cache (frequently accessed result sets)
//!
//! Each level has configurable capacity and TTL. Cache hit rates are
//! tracked per level for observability.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Cache entry with TTL and access tracking.
#[derive(Debug, Clone)]
struct CacheEntry<V: Clone> {
    value: V,
    inserted_at: Instant,
    access_count: u64,
}

impl<V: Clone> CacheEntry<V> {
    fn is_expired(&self, ttl: Duration) -> bool {
        self.inserted_at.elapsed() >= ttl
    }
}

/// Per-level cache statistics.
#[derive(Debug, Clone, PartialEq)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: usize,
    pub capacity: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// Aggregated stats for all cache levels.
#[derive(Debug, Clone)]
pub struct MultiLevelCacheStats {
    pub l1: CacheStats,
    pub l2: CacheStats,
    pub l3: CacheStats,
}

// ── L1: Exact request hash cache ─────────────────────────────────────────────

struct L1Cache<V: Clone> {
    entries: HashMap<u64, CacheEntry<V>>,
    capacity: usize,
    ttl: Duration,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl<V: Clone> L1Cache<V> {
    fn new(capacity: usize, ttl: Duration) -> Self {
        Self {
            entries: HashMap::with_capacity(capacity),
            capacity,
            ttl,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    fn get(&mut self, key: u64) -> Option<V> {
        if let Some(entry) = self.entries.get_mut(&key) {
            if !entry.is_expired(self.ttl) {
                entry.access_count += 1;
                self.hits.fetch_add(1, Ordering::Relaxed);
                return Some(entry.value.clone());
            }
            self.entries.remove(&key);
        }
        self.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    fn insert(&mut self, key: u64, value: V) {
        if self.entries.len() >= self.capacity {
            self.evict_oldest();
        }
        self.entries.insert(
            key,
            CacheEntry {
                value,
                inserted_at: Instant::now(),
                access_count: 0,
            },
        );
    }

    fn evict_expired(&mut self) {
        let ttl = self.ttl;
        self.entries.retain(|_, entry| !entry.is_expired(ttl));
    }

    fn evict_oldest(&mut self) {
        if let Some((&oldest_key, _)) = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.inserted_at)
        {
            self.entries.remove(&oldest_key);
        }
    }

    fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            size: self.entries.len(),
            capacity: self.capacity,
        }
    }
}

// ── L3: Top-k result cache ──────────────────────────────────────────────────

struct L3Cache {
    entries: HashMap<u64, CacheEntry<Vec<u64>>>,
    capacity: usize,
    ttl: Duration,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl L3Cache {
    fn new(capacity: usize, ttl: Duration) -> Self {
        Self {
            entries: HashMap::with_capacity(capacity),
            capacity,
            ttl,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    fn get(&mut self, key: u64) -> Option<Vec<u64>> {
        if let Some(entry) = self.entries.get_mut(&key) {
            if !entry.is_expired(self.ttl) {
                entry.access_count += 1;
                self.hits.fetch_add(1, Ordering::Relaxed);
                return Some(entry.value.clone());
            }
            self.entries.remove(&key);
        }
        self.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    fn insert(&mut self, key: u64, value: Vec<u64>) {
        if self.entries.len() >= self.capacity {
            self.evict_lru();
        }
        self.entries.insert(
            key,
            CacheEntry {
                value,
                inserted_at: Instant::now(),
                access_count: 0,
            },
        );
    }

    fn evict_lru(&mut self) {
        if let Some((&lru_key, _)) = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.access_count)
        {
            self.entries.remove(&lru_key);
        }
    }

    fn evict_expired(&mut self) {
        let ttl = self.ttl;
        self.entries.retain(|_, entry| !entry.is_expired(ttl));
    }

    fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            size: self.entries.len(),
            capacity: self.capacity,
        }
    }
}

// ── Multi-Level Cache ────────────────────────────────────────────────────────

/// Configuration for multi-level cache.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// L1 capacity (exact match cache).
    pub l1_capacity: usize,
    /// L1 TTL.
    pub l1_ttl: Duration,
    /// L3 capacity (top-k result cache).
    pub l3_capacity: usize,
    /// L3 TTL.
    pub l3_ttl: Duration,
    /// Cosine similarity threshold for L2 (0.0–1.0). Queries with
    /// similarity above this threshold to a cached query can reuse
    /// its results.
    pub l2_similarity_threshold: f64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            l1_capacity: 2048,
            l1_ttl: Duration::from_secs(300), // 5 min
            l3_capacity: 512,
            l3_ttl: Duration::from_secs(600), // 10 min
            l2_similarity_threshold: 0.98,
        }
    }
}

/// Multi-level cache for pipeline responses (FT-024).
///
/// Provides three caching levels:
/// - L1: exact request hash → cached output
/// - L2: embedding similarity → reuse results from similar queries
/// - L3: top-k result sets → frequently accessed result lists
///
/// Thread-safe via `parking_lot::Mutex`.
pub struct MultiLevelCache {
    l1: parking_lot::Mutex<L1Cache<Vec<u8>>>,
    /// L2: stored query embeddings for similarity matching.
    l2_embeddings: parking_lot::Mutex<HashMap<u64, Vec<f64>>>,
    l2_result_keys: parking_lot::Mutex<HashMap<u64, u64>>,
    l2_hits: AtomicU64,
    l2_misses: AtomicU64,
    l3: parking_lot::Mutex<L3Cache>,
    l2_threshold: f64,
}

impl MultiLevelCache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            l1: parking_lot::Mutex::new(L1Cache::new(config.l1_capacity, config.l1_ttl)),
            l2_embeddings: parking_lot::Mutex::new(HashMap::new()),
            l2_result_keys: parking_lot::Mutex::new(HashMap::new()),
            l2_hits: AtomicU64::new(0),
            l2_misses: AtomicU64::new(0),
            l3: parking_lot::Mutex::new(L3Cache::new(config.l3_capacity, config.l3_ttl)),
            l2_threshold: config.l2_similarity_threshold,
        }
    }

    /// L1: Get a cached value by exact request hash.
    pub fn l1_get(&self, request_hash: u64) -> Option<Vec<u8>> {
        self.l1.lock().get(request_hash)
    }

    /// L1: Insert a value by request hash.
    pub fn l1_insert(&self, request_hash: u64, value: Vec<u8>) {
        self.l1.lock().insert(request_hash, value);
    }

    /// L2: Find a cached result key for a similar query embedding.
    ///
    /// Returns the result key if a cached embedding has cosine similarity
    /// above the configured threshold.
    pub fn l2_lookup(&self, query_embedding: &[f64]) -> Option<u64> {
        let embeddings = self.l2_embeddings.lock();
        let result_keys = self.l2_result_keys.lock();

        for (&key, cached_emb) in embeddings.iter() {
            if let Some(similarity) = cosine_similarity(query_embedding, cached_emb) {
                if similarity >= self.l2_threshold {
                    if let Some(&result_key) = result_keys.get(&key) {
                        self.l2_hits.fetch_add(1, Ordering::Relaxed);
                        return Some(result_key);
                    }
                }
            }
        }
        self.l2_misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    /// L2: Store a query embedding and its result key for future similarity matching.
    pub fn l2_insert(&self, query_hash: u64, query_embedding: &[f64], result_key: u64) {
        let mut embeddings = self.l2_embeddings.lock();
        let mut result_keys = self.l2_result_keys.lock();
        // Cap at 4096 entries — evict the smallest key (deterministic).
        if embeddings.len() >= 4096 {
            if let Some((&evict_key, _)) = embeddings.iter().min_by_key(|(&k, _)| k) {
                embeddings.remove(&evict_key);
                result_keys.remove(&evict_key);
            }
        }
        embeddings.insert(query_hash, query_embedding.to_vec());
        result_keys.insert(query_hash, result_key);
    }

    /// L3: Get cached top-k results by key.
    pub fn l3_get(&self, key: u64) -> Option<Vec<u64>> {
        self.l3.lock().get(key)
    }

    /// L3: Store top-k results.
    pub fn l3_insert(&self, key: u64, result_ids: Vec<u64>) {
        self.l3.lock().insert(key, result_ids);
    }

    /// Evict expired entries from all levels.
    pub fn evict_expired(&self) {
        self.l1.lock().evict_expired();
        self.l3.lock().evict_expired();
    }

    /// Get aggregated stats for all levels.
    pub fn stats(&self) -> MultiLevelCacheStats {
        let l1 = self.l1.lock().stats();
        let l3 = self.l3.lock().stats();

        let l2 = CacheStats {
            hits: self.l2_hits.load(Ordering::Relaxed),
            misses: self.l2_misses.load(Ordering::Relaxed),
            size: self.l2_embeddings.lock().len(),
            capacity: 4096,
        };

        MultiLevelCacheStats { l1, l2, l3 }
    }
}

/// Compute cosine similarity between two vectors.
fn cosine_similarity(a: &[f64], b: &[f64]) -> Option<f64> {
    if a.len() != b.len() || a.is_empty() {
        return None;
    }
    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom < 1e-15 {
        None
    } else {
        Some(dot / denom)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn l1_exact_hit() {
        let cache = MultiLevelCache::new(CacheConfig::default());
        cache.l1_insert(42, vec![1, 2, 3]);
        assert_eq!(cache.l1_get(42), Some(vec![1, 2, 3]));
        assert_eq!(cache.l1_get(99), None);
    }

    #[test]
    fn l1_miss_returns_none() {
        let cache = MultiLevelCache::new(CacheConfig::default());
        assert_eq!(cache.l1_get(1), None);
    }

    #[test]
    fn l2_similar_query_hit() {
        let cache = MultiLevelCache::new(CacheConfig {
            l2_similarity_threshold: 0.95,
            ..Default::default()
        });
        let emb1 = vec![1.0, 0.0, 0.0];
        cache.l2_insert(1, &emb1, 100);

        // Nearly identical query
        let emb2 = vec![0.999, 0.001, 0.0];
        let result = cache.l2_lookup(&emb2);
        assert_eq!(result, Some(100));
    }

    #[test]
    fn l2_dissimilar_query_miss() {
        let cache = MultiLevelCache::new(CacheConfig {
            l2_similarity_threshold: 0.95,
            ..Default::default()
        });
        cache.l2_insert(1, &[1.0, 0.0, 0.0], 100);

        // Very different query
        let result = cache.l2_lookup(&[0.0, 0.0, 1.0]);
        assert_eq!(result, None);
    }

    #[test]
    fn l3_topk_cache() {
        let cache = MultiLevelCache::new(CacheConfig::default());
        cache.l3_insert(10, vec![1, 2, 3, 4, 5]);
        assert_eq!(cache.l3_get(10), Some(vec![1, 2, 3, 4, 5]));
        assert_eq!(cache.l3_get(11), None);
    }

    #[test]
    fn stats_tracking() {
        let cache = MultiLevelCache::new(CacheConfig::default());
        cache.l1_insert(1, vec![1]);
        cache.l1_get(1); // hit
        cache.l1_get(2); // miss

        let stats = cache.stats();
        assert_eq!(stats.l1.hits, 1);
        assert_eq!(stats.l1.misses, 1);
        assert_eq!(stats.l1.size, 1);
    }

    #[test]
    fn cache_stats_hit_rate() {
        let stats = CacheStats {
            hits: 3,
            misses: 1,
            size: 0,
            capacity: 0,
        };
        assert!((stats.hit_rate() - 0.75).abs() < 1e-10);
    }

    #[test]
    fn cache_stats_zero_hit_rate() {
        let stats = CacheStats {
            hits: 0,
            misses: 0,
            size: 0,
            capacity: 0,
        };
        assert!((stats.hit_rate() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn cosine_similarity_identical() {
        let sim = cosine_similarity(&[1.0, 0.0], &[1.0, 0.0]);
        assert!(sim.is_some());
        assert!((sim.unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cosine_similarity_orthogonal() {
        let sim = cosine_similarity(&[1.0, 0.0], &[0.0, 1.0]);
        assert!(sim.is_some());
        assert!(sim.unwrap().abs() < 1e-10);
    }

    #[test]
    fn cosine_similarity_different_lengths() {
        assert!(cosine_similarity(&[1.0], &[1.0, 0.0]).is_none());
    }
}

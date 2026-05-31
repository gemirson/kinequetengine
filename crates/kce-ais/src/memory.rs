//! Antigen memory — persistent memory of known threat patterns.
//!
//! Stores previously detected anomalies with metadata for future
//! pattern matching. Inspired by the biological immune system's
//! memory B-cells that remember past infections.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Status of an antigen record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AntigenStatus {
    /// The antigen is active and will be used for matching.
    #[default]
    Active,
    /// The antigen has expired beyond the configured retention window.
    Expired,
}

/// A stored antigen pattern with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntigenRecord {
    /// Unique identifier for this antigen.
    pub id: u64,
    /// The pattern vector.
    pub pattern: Vec<f64>,
    /// How many times this antigen has been encountered.
    pub encounter_count: u64,
    /// First seen timestamp (epoch ms).
    pub first_seen_ms: u64,
    /// Last seen timestamp (epoch ms).
    pub last_seen_ms: u64,
    /// Severity classification.
    pub severity: Severity,
    /// Current status of the antigen (active or expired).
    pub status: AntigenStatus,
}

/// Severity levels for detected antigens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Low severity — informational.
    Low,
    /// Medium severity — requires attention.
    Medium,
    /// High severity — immediate action needed.
    High,
    /// Critical severity — system-threatening.
    Critical,
}

/// Configuration for antigen memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Maximum number of antigens to store.
    pub max_entries: usize,
    /// Similarity threshold for matching (L2 distance).
    pub match_threshold: f64,
    /// Number of days after which an antigen expires.
    /// A value of 0 means antigens never expire.
    pub expiry_days: u32,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_entries: 10_000,
            match_threshold: 0.1,
            expiry_days: 90,
        }
    }
}

/// Antigen memory store.
#[derive(Serialize, Deserialize)]
pub struct AntigenMemory {
    records: HashMap<u64, AntigenRecord>,
    next_id: u64,
    config: MemoryConfig,
}

impl AntigenMemory {
    /// Create a new antigen memory store.
    pub fn new(config: MemoryConfig) -> Self {
        Self {
            records: HashMap::new(),
            next_id: 1,
            config,
        }
    }

    /// Export memory state as a JSON byte vector.
    pub fn export_state(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| format!("serialize error: {e}"))
    }

    /// Import memory state from a JSON byte vector.
    pub fn import_state(&mut self, state: &[u8]) -> Result<(), String> {
        let imported: AntigenMemory =
            serde_json::from_slice(state).map_err(|e| format!("deserialize error: {e}"))?;
        self.records = imported.records;
        self.next_id = imported.next_id;
        self.config = imported.config;
        Ok(())
    }

    /// Store a new antigen pattern.
    ///
    /// Returns the assigned ID. If the memory is full, the oldest
    /// (lowest encounter count) entry is evicted.
    pub fn store(&mut self, pattern: Vec<f64>, severity: Severity, now_ms: u64) -> u64 {
        // Evict if full
        if self.records.len() >= self.config.max_entries {
            if let Some(min_id) = self
                .records
                .iter()
                .min_by_key(|(_, r)| r.encounter_count)
                .map(|(id, _)| *id)
            {
                self.records.remove(&min_id);
            }
        }

        let id = self.next_id;
        self.next_id += 1;
        self.records.insert(
            id,
            AntigenRecord {
                id,
                pattern,
                encounter_count: 1,
                first_seen_ms: now_ms,
                last_seen_ms: now_ms,
                severity,
                status: AntigenStatus::Active,
            },
        );
        id
    }

    /// Check whether an antigen record is expired based on the configured
    /// `expiry_days` and the current timestamp.
    ///
    /// Returns `true` if the antigen has exceeded its retention window.
    /// When `expiry_days == 0`, antigens never expire.
    pub fn is_expired(record: &AntigenRecord, now_ms: u64, expiry_days: u32) -> bool {
        if expiry_days == 0 {
            return false;
        }
        let expiry_ms = (expiry_days as u64) * 86_400 * 1000;
        now_ms.saturating_sub(record.first_seen_ms) > expiry_ms
    }

    /// Try to match a vector against stored antigens.
    ///
    /// Expired antigens are skipped. Returns the ID of the closest active
    /// match if within threshold, or `None`.
    pub fn match_pattern(&self, vector: &[f64], now_ms: u64) -> Option<u64> {
        let mut best_id = None;
        let mut best_dist = self.config.match_threshold;

        for record in self.records.values() {
            if Self::is_expired(record, now_ms, self.config.expiry_days) {
                continue;
            }

            if record.pattern.len() != vector.len() {
                continue;
            }

            let dist: f64 = vector
                .iter()
                .zip(record.pattern.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                .sqrt();

            if dist < best_dist {
                best_dist = dist;
                best_id = Some(record.id);
            }
        }

        best_id
    }

    /// Record a re-encounter of an antigen.
    pub fn record_encounter(&mut self, id: u64, now_ms: u64) {
        if let Some(record) = self.records.get_mut(&id) {
            record.encounter_count += 1;
            record.last_seen_ms = now_ms;
        }
    }

    /// Get a reference to a stored antigen.
    pub fn get(&self, id: u64) -> Option<&AntigenRecord> {
        self.records.get(&id)
    }

    /// Get the number of stored antigens.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Check if memory is empty.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Mark all antigens older than the configured expiry window as `Expired`.
    ///
    /// When `expiry_days == 0` (never expires), this is a no-op.
    pub fn expire_old(&mut self, now_ms: u64) {
        let expiry_days = self.config.expiry_days;
        for record in self.records.values_mut() {
            if Self::is_expired(record, now_ms, expiry_days) {
                record.status = AntigenStatus::Expired;
            }
        }
    }

    /// Save all antigen records to a JSON file.
    ///
    /// Overwrites the file if it exists. Returns the number of records written.
    ///
    /// # Errors
    ///
    /// Returns a `serde_json::Error` if serialization fails or `std::io::Error`
    /// if the file cannot be written.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<usize, String> {
        let records: Vec<&AntigenRecord> = self.records.values().collect();
        let count = records.len();
        let json = serde_json::to_string_pretty(&records)
            .map_err(|e| format!("serialize error: {e}"))?;
        fs::write(path, json).map_err(|e| format!("write error: {e}"))?;
        Ok(count)
    }

    /// Load antigen records from a JSON file, merging them into the store.
    ///
    /// Existing records are not overwritten. Loaded records that conflict
    /// with existing IDs are skipped. Returns the number of records loaded.
    ///
    /// # Errors
    ///
    /// Returns an error string if the file cannot be read or parsed.
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, String> {
        let json = fs::read_to_string(path).map_err(|e| format!("read error: {e}"))?;
        let records: Vec<AntigenRecord> =
            serde_json::from_str(&json).map_err(|e| format!("parse error: {e}"))?;
        let mut loaded = 0usize;
        for record in records {
            if !self.records.contains_key(&record.id) {
                if record.id >= self.next_id {
                    self.next_id = record.id + 1;
                }
                self.records.insert(record.id, record);
                loaded += 1;
            }
        }
        Ok(loaded)
    }

    /// Export all antigen records as a pretty-printed JSON string.
    pub fn export_json(&self) -> String {
        let records: Vec<&AntigenRecord> = self.records.values().collect();
        serde_json::to_string_pretty(&records).unwrap_or_else(|_| "[]".into())
    }

    /// Export all antigen records as CSV.
    pub fn export_csv(&self) -> String {
        let mut csv = String::from(
            "antigen_id,severity,first_seen_ms,last_seen_ms,encounter_count,status\n",
        );
        for record in self.records.values() {
            let status = match record.status {
                AntigenStatus::Active => "ACTIVE",
                AntigenStatus::Expired => "EXPIRED",
            };
            csv.push_str(&format!(
                "{},{:?},{},{},{},{}\n",
                record.id,
                record.severity,
                record.first_seen_ms,
                record.last_seen_ms,
                record.encounter_count,
                status
            ));
        }
        csv
    }
}

impl Default for AntigenMemory {
    fn default() -> Self {
        Self::new(MemoryConfig::default())
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn store_and_retrieve() {
        let mut mem = AntigenMemory::default();
        let id = mem.store(vec![1.0, 2.0], Severity::High, 1000);
        let record = mem.get(id).unwrap();
        assert_eq!(record.encounter_count, 1);
        assert_eq!(record.severity, Severity::High);
        assert_eq!(record.status, AntigenStatus::Active);
    }

    #[test]
    fn match_returns_closest() {
        let mut mem = AntigenMemory::default();
        mem.store(vec![0.0, 0.0], Severity::Low, 1000);
        let id2 = mem.store(vec![1.0, 0.0], Severity::Medium, 1000);
        let matched = mem.match_pattern(&[0.9, 0.0], 1000);
        assert_eq!(matched, Some(id2));
    }

    #[test]
    fn match_returns_none_if_too_far() {
        let mut mem = AntigenMemory::default();
        mem.store(vec![0.0, 0.0], Severity::Low, 1000);
        assert!(mem.match_pattern(&[10.0, 10.0], 1000).is_none());
    }

    #[test]
    fn encounter_increments_count() {
        let mut mem = AntigenMemory::default();
        let id = mem.store(vec![1.0], Severity::Low, 1000);
        mem.record_encounter(id, 2000);
        let record = mem.get(id).unwrap();
        assert_eq!(record.encounter_count, 2);
        assert_eq!(record.last_seen_ms, 2000);
    }

    #[test]
    fn eviction_on_full() {
        let config = MemoryConfig {
            max_entries: 2,
            match_threshold: 0.1,
            expiry_days: 90,
        };
        let mut mem = AntigenMemory::new(config);
        mem.store(vec![0.0], Severity::Low, 1000);
        mem.store(vec![1.0], Severity::Low, 1000);
        mem.store(vec![2.0], Severity::Low, 1000); // should evict one
        assert_eq!(mem.len(), 2);
    }

    /// 91 days in ms = 91 * 86400 * 1000 = 7_862_400_000
    const NINETY_ONE_DAYS_MS: u64 = 91 * 86_400 * 1000;

    #[test]
    fn test_antigen_expiry() {
        let mut mem = AntigenMemory::default(); // expiry_days = 90
        let id = mem.store(vec![1.0, 2.0], Severity::High, 0);

        // At 1 ms, antigen is still active
        assert!(!AntigenMemory::is_expired(
            mem.get(id).unwrap(),
            1,
            90
        ));

        // At exactly the boundary it is still active (>, not >=)
        let boundary_ms = 90_u64 * 86_400 * 1000;
        assert!(!AntigenMemory::is_expired(
            mem.get(id).unwrap(),
            boundary_ms,
            90
        ));

        // After 91 days it is expired
        assert!(AntigenMemory::is_expired(
            mem.get(id).unwrap(),
            NINETY_ONE_DAYS_MS,
            90
        ));

        // expire_old marks the record
        mem.expire_old(NINETY_ONE_DAYS_MS);
        assert_eq!(mem.get(id).unwrap().status, AntigenStatus::Expired);
    }

    #[test]
    fn test_match_skips_expired() {
        let config = MemoryConfig {
            max_entries: 10,
            match_threshold: 1.5,
            expiry_days: 90,
        };
        let mut mem = AntigenMemory::new(config);
        let id1 = mem.store(vec![0.0, 0.0], Severity::Low, 0); // old
        let id2 = mem.store(vec![0.3, 0.0], Severity::Low, NINETY_ONE_DAYS_MS); // recent, within threshold 0.5

        // Both are active, id1 is closer
        assert_eq!(mem.match_pattern(&[0.0, 0.0], 1000), Some(id1));

        // Expire old records at 91 days
        mem.expire_old(NINETY_ONE_DAYS_MS);
        assert_eq!(mem.get(id1).unwrap().status, AntigenStatus::Expired);
        assert_eq!(mem.get(id2).unwrap().status, AntigenStatus::Active);

        // Now match skips id1 (expired) and returns id2 (still active, within threshold)
        let matched = mem.match_pattern(&[0.0, 0.0], NINETY_ONE_DAYS_MS);
        assert_eq!(matched, Some(id2));
    }

    #[test]
    fn test_export_json_valid() {
        let mut mem = AntigenMemory::default();
        mem.store(vec![1.0, 2.0], Severity::High, 1000);
        mem.store(vec![3.0, 4.0], Severity::Low, 2000);

        let json_str = mem.export_json();
        let parsed: Vec<AntigenRecord> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn test_export_csv_format() {
        let mut mem = AntigenMemory::default();
        mem.store(vec![1.0], Severity::High, 1000);
        mem.store(vec![2.0], Severity::Low, 2000);

        let csv = mem.export_csv();
        let mut lines = csv.lines();
        assert_eq!(
            lines.next().unwrap(),
            "antigen_id,severity,first_seen_ms,last_seen_ms,encounter_count,status"
        );
        // Should have exactly 2 data rows + 1 header
        let data_lines: Vec<&str> = lines.collect();
        assert_eq!(data_lines.len(), 2);
        // Each data line should end with ACTIVE
        for line in &data_lines {
            assert!(line.ends_with(",ACTIVE"), "expected ACTIVE status in: {line}");
        }
    }

    #[test]
    fn test_expiry_disabled() {
        let config = MemoryConfig {
            max_entries: 10,
            match_threshold: 0.1,
            expiry_days: 0, // never expire
        };
        let mut mem = AntigenMemory::new(config);
        let id = mem.store(vec![1.0], Severity::Low, 0);

        // Even after an absurd amount of time, it never expires
        assert!(!AntigenMemory::is_expired(
            mem.get(id).unwrap(),
            u64::MAX / 2,
            0
        ));

        mem.expire_old(u64::MAX / 2);
        assert_eq!(mem.get(id).unwrap().status, AntigenStatus::Active);
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let path = "/tmp/kce_ais_test_roundtrip.json";
        let mut mem = AntigenMemory::default();
        mem.store(vec![1.0, 2.0], Severity::High, 1000);
        mem.store(vec![3.0, 4.0], Severity::Low, 2000);

        let saved = mem.save_to_file(path).unwrap();
        assert_eq!(saved, 2);

        let mut mem2 = AntigenMemory::default();
        let loaded = mem2.load_from_file(path).unwrap();
        assert_eq!(loaded, 2);
        assert_eq!(mem2.len(), 2);

        // Verify data matches
        let r1 = mem2.get(1).unwrap();
        assert_eq!(r1.pattern, vec![1.0, 2.0]);
        assert_eq!(r1.severity, Severity::High);

        let r2 = mem2.get(2).unwrap();
        assert_eq!(r2.pattern, vec![3.0, 4.0]);
        assert_eq!(r2.severity, Severity::Low);

        // Cleanup
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_load_skips_existing_ids() {
        let path = "/tmp/kce_ais_test_skip.json";
        let mut mem = AntigenMemory::default();
        mem.store(vec![1.0], Severity::High, 1000);
        mem.save_to_file(path).unwrap();

        // mem2 already has id=1
        let mut mem2 = AntigenMemory::default();
        mem2.store(vec![9.0], Severity::Low, 500);
        let loaded = mem2.load_from_file(path).unwrap();
        assert_eq!(loaded, 0); // skipped, id=1 already exists
        assert_eq!(mem2.len(), 1);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_load_from_nonexistent_file() {
        let mut mem = AntigenMemory::default();
        let result = mem.load_from_file("/tmp/kce_nonexistent_file_12345.json");
        assert!(result.is_err());
    }
}

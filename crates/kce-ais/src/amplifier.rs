//! Response amplifier — boosts priority for critical contexts.
//!
//! When the immune system detects a critical anomaly, the amplifier
//! increases the priority of the associated mRNA instruction so that
//! it gets processed first.

use crate::memory::Severity;

/// Return the numeric multiplier for the given severity level.
fn severity_multiplier(severity: Severity) -> f64 {
    match severity {
        Severity::Low => 1.5,
        Severity::Medium => 2.0,
        Severity::High => 3.0,
        Severity::Critical => 5.0,
    }
}

/// Amplification trigger conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmplifyTrigger {
    /// Anomaly detected above severity threshold.
    AnomalyDetected(Severity),
    /// Repeated antigen encounter (memory hit).
    MemoryHit(Severity),
    /// Network regulation flagged critical state.
    RegulationAlert(Severity),
}

/// Configuration for the response amplifier.
#[derive(Debug, Clone)]
pub struct AmplifierConfig {
    /// Priority boost amount per trigger.
    pub boost_per_trigger: u8,
    /// Maximum priority cap (10).
    pub max_priority: u8,
    /// Decay rate per cycle (priority decreases by this amount when inactive).
    pub decay_per_cycle: u8,
}

impl Default for AmplifierConfig {
    fn default() -> Self {
        Self {
            boost_per_trigger: 2,
            max_priority: 10,
            decay_per_cycle: 1,
        }
    }
}

/// Amplifier that tracks and boosts priority for specific contexts.
pub struct ResponseAmplifier {
    /// Current amplification levels per context ID.
    levels: std::collections::HashMap<u64, u8>,
    config: AmplifierConfig,
}

impl ResponseAmplifier {
    /// Create a new amplifier with default config.
    pub fn new(config: AmplifierConfig) -> Self {
        Self {
            levels: std::collections::HashMap::new(),
            config,
        }
    }

    /// Record a trigger event for a context, boosting its priority.
    ///
    /// The severity embedded in the trigger is used to scale
    /// `boost_per_trigger` via a multiplier:
    /// Low=1.5x, Medium=2x, High=3x, Critical=5x.
    pub fn trigger(&mut self, context_id: u64, trigger: AmplifyTrigger) {
        let severity = match &trigger {
            AmplifyTrigger::AnomalyDetected(s)
            | AmplifyTrigger::MemoryHit(s)
            | AmplifyTrigger::RegulationAlert(s) => *s,
        };
        let multiplier = severity_multiplier(severity);
        let effective_boost = (self.config.boost_per_trigger as f64 * multiplier).round() as u8;
        let level = self.levels.entry(context_id).or_insert(0);
        *level = (*level + effective_boost).min(self.config.max_priority);
    }

    /// Get the current amplification level for a context.
    pub fn get_level(&self, context_id: u64) -> u8 {
        self.levels.get(&context_id).copied().unwrap_or(0)
    }

    /// Apply decay to all contexts. Removes entries that reach zero.
    pub fn decay_all(&mut self) {
        let decay = self.config.decay_per_cycle;
        self.levels.retain(|_, level| {
            *level = level.saturating_sub(decay);
            *level > 0
        });
    }

    /// Get the number of actively amplified contexts.
    pub fn active_count(&self) -> usize {
        self.levels.len()
    }
}

impl Default for ResponseAmplifier {
    fn default() -> Self {
        Self::new(AmplifierConfig::default())
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn trigger_increases_level() {
        let mut amp = ResponseAmplifier::default();
        // Low severity: boost=2 * 1.5 = 3
        amp.trigger(1, AmplifyTrigger::AnomalyDetected(Severity::Low));
        assert_eq!(amp.get_level(1), 3);
    }

    #[test]
    fn trigger_caps_at_max() {
        let mut amp = ResponseAmplifier::default();
        for _ in 0..10 {
            amp.trigger(1, AmplifyTrigger::AnomalyDetected(Severity::Medium));
        }
        assert_eq!(amp.get_level(1), 10);
    }

    #[test]
    fn decay_reduces_level() {
        let mut amp = ResponseAmplifier::default();
        amp.trigger(1, AmplifyTrigger::AnomalyDetected(Severity::Low)); // +3
        amp.decay_all(); // 3 - 1 = 2
        assert_eq!(amp.get_level(1), 2);
    }

    #[test]
    fn decay_removes_zero_entries() {
        let mut amp = ResponseAmplifier::new(AmplifierConfig {
            boost_per_trigger: 1,
            max_priority: 10,
            decay_per_cycle: 1,
        });
        amp.trigger(1, AmplifyTrigger::AnomalyDetected(Severity::Low)); // 1 * 1.5 = 1.5 -> 2
        // Need two cycles of decay to remove since round gives 2
        amp.decay_all(); // 2 - 1 = 1
        amp.decay_all(); // 1 - 1 = 0 -> removed
        assert_eq!(amp.active_count(), 0);
    }

    #[test]
    fn inactive_context_returns_zero() {
        let amp = ResponseAmplifier::default();
        assert_eq!(amp.get_level(999), 0);
    }

    #[test]
    fn test_low_severity_multiplier() {
        let mut amp = ResponseAmplifier::default();
        amp.trigger(1, AmplifyTrigger::AnomalyDetected(Severity::Low));
        // boost_per_trigger=2 * 1.5 = 3.0 -> 3
        assert_eq!(amp.get_level(1), 3);
    }

    #[test]
    fn test_critical_severity_multiplier() {
        let mut amp = ResponseAmplifier::default();
        amp.trigger(1, AmplifyTrigger::AnomalyDetected(Severity::Critical));
        // boost_per_trigger=2 * 5.0 = 10
        assert_eq!(amp.get_level(1), 10);
    }

    #[test]
    fn test_cap_respected_with_critical() {
        let mut amp = ResponseAmplifier::default();
        // First critical trigger: 2 * 5.0 = 10 (hitting max)
        amp.trigger(1, AmplifyTrigger::AnomalyDetected(Severity::Critical));
        assert_eq!(amp.get_level(1), 10);
        // Second critical trigger should not exceed max
        amp.trigger(1, AmplifyTrigger::MemoryHit(Severity::Critical));
        assert_eq!(amp.get_level(1), 10);
    }

    #[test]
    fn severity_respects_max_priority() {
        let mut amp = ResponseAmplifier::default();
        amp.trigger(1, AmplifyTrigger::AnomalyDetected(Severity::Medium)); // 2 * 2.0 = 4
        amp.trigger(1, AmplifyTrigger::MemoryHit(Severity::Critical));     // 2 * 5.0 = 10, total=14, capped to 10
        assert_eq!(amp.get_level(1), 10);
    }

    #[test]
    fn severity_level_multipliers() {
        assert!((severity_multiplier(Severity::Low) - 1.5).abs() < 1e-10);
        assert!((severity_multiplier(Severity::Medium) - 2.0).abs() < 1e-10);
        assert!((severity_multiplier(Severity::High) - 3.0).abs() < 1e-10);
        assert!((severity_multiplier(Severity::Critical) - 5.0).abs() < 1e-10);
    }
}

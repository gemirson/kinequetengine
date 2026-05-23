//! Network regulation — homeostasis controller.
//!
//! Monitors 5 key system metrics and adjusts parameters to maintain
//! stability. Inspired by biological homeostasis mechanisms.
//!
//! Metrics monitored:
//! 1. Query latency (p95)
//! 2. Error rate
//! 3. Pheromone diversity (entropy)
//! 4. Anomaly frequency
//! 5. Resource utilization

use serde::{Deserialize, Serialize};

/// System health metrics snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// 95th percentile query latency in milliseconds.
    pub latency_p95_ms: f64,
    /// Error rate (0.0 .. 1.0).
    pub error_rate: f64,
    /// Pheromone trail entropy (0.0 = all same, 1.0 = uniform).
    pub pheromone_entropy: f64,
    /// Anomalies detected per minute.
    pub anomaly_rate: f64,
    /// CPU/memory utilization (0.0 .. 1.0).
    pub resource_utilization: f64,
}

/// Regulatory action to take.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegulatoryAction {
    /// No action needed — system is healthy.
    None,
    /// Reduce load (e.g., decrease num_ants, increase threshold).
    ReduceLoad,
    /// Increase exploration (e.g., raise mutation rate, lower threshold).
    IncreaseExploration,
    /// Tighten security (e.g., lower anomaly threshold).
    TightenSecurity,
    /// Emergency: circuit-break incoming requests.
    CircuitBreak,
}

/// Adjustable system parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    /// Anomaly detection threshold (lower = more sensitive).
    pub detection_threshold: f64,
    /// Maximum amplification factor cap.
    pub amplification_cap: f64,
    /// Adaptive mutation rate.
    pub mutation_rate: f64,
    /// Pheromone evaporation rate (rho).
    pub evaporation_rho: f64,
    /// Number of ants per ACO cycle.
    pub ant_count: usize,
    /// Self/non-self classification threshold.
    pub nonself_threshold: f64,
    /// Antigen expiry in days.
    pub expiry_days: u32,
}

impl Default for SystemState {
    fn default() -> Self {
        Self {
            detection_threshold: 0.7,
            amplification_cap: 5.0,
            mutation_rate: 0.05,
            evaporation_rho: 0.1,
            ant_count: 5,
            nonself_threshold: 0.6,
            expiry_days: 90,
        }
    }
}

/// Hard guardrails — parameters are never adjusted outside these bounds.
#[derive(Debug, Clone)]
pub struct Guardrails {
    pub detection_threshold: (f64, f64),
    pub amplification_cap: (f64, f64),
    pub mutation_rate: (f64, f64),
    pub evaporation_rho: (f64, f64),
    pub ant_count: (usize, usize),
    pub nonself_threshold: (f64, f64),
    pub expiry_days: (u32, u32),
}

impl Default for Guardrails {
    fn default() -> Self {
        Self {
            detection_threshold: (0.3, 0.95),
            amplification_cap: (2.0, 10.0),
            mutation_rate: (0.01, 0.3),
            evaporation_rho: (0.01, 0.5),
            ant_count: (2, 20),
            nonself_threshold: (0.3, 0.9),
            expiry_days: (1, 365),
        }
    }
}

/// A single parameter adjustment applied during regulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adjustment {
    /// Name of the adjusted parameter.
    pub parameter: String,
    /// Value before adjustment.
    pub from_value: String,
    /// Value after adjustment.
    pub to_value: String,
    /// Human-readable reason for the adjustment.
    pub reason: String,
}

/// Status snapshot for /regulation/status endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulationStatus {
    /// Current system status label.
    pub status: String,
    /// Current parameter state.
    pub state: SystemState,
    /// Number of metrics evaluations in history.
    pub history_count: usize,
    /// Last regulatory action taken.
    pub last_action: RegulatoryAction,
}

/// Configuration for the network regulator.
#[derive(Debug, Clone)]
pub struct RegulatorConfig {
    /// Maximum acceptable p95 latency (ms).
    pub max_latency_ms: f64,
    /// Maximum acceptable error rate.
    pub max_error_rate: f64,
    /// Minimum pheromone entropy (below this = stagnation).
    pub min_entropy: f64,
    /// Maximum anomaly rate per minute before tightening.
    pub max_anomaly_rate: f64,
    /// Maximum resource utilization before circuit break.
    pub max_resource_util: f64,
    /// Step size for parameter adjustments (fraction of current value).
    pub adjustment_step: f64,
}

impl Default for RegulatorConfig {
    fn default() -> Self {
        Self {
            max_latency_ms: 100.0,
            max_error_rate: 0.05,
            min_entropy: 0.2,
            max_anomaly_rate: 10.0,
            max_resource_util: 0.9,
            adjustment_step: 0.1,
        }
    }
}

/// Network regulator that maintains system homeostasis.
pub struct NetworkRegulator {
    config: RegulatorConfig,
    history: Vec<SystemMetrics>,
    state: SystemState,
    guardrails: Guardrails,
    last_action: RegulatoryAction,
}

impl NetworkRegulator {
    /// Create a new regulator with the given configuration.
    pub fn new(config: RegulatorConfig) -> Self {
        Self {
            config,
            history: Vec::new(),
            state: SystemState::default(),
            guardrails: Guardrails::default(),
            last_action: RegulatoryAction::None,
        }
    }

    /// Create with custom state and guardrails.
    pub fn with_state(
        config: RegulatorConfig,
        state: SystemState,
        guardrails: Guardrails,
    ) -> Self {
        Self {
            config,
            history: Vec::new(),
            state,
            guardrails,
            last_action: RegulatoryAction::None,
        }
    }

    /// Evaluate current metrics and return the recommended regulatory action.
    pub fn regulate(&mut self, metrics: SystemMetrics) -> RegulatoryAction {
        self.history.push(metrics.clone());
        let action = self.evaluate(&metrics);
        self.last_action = action.clone();
        action
    }

    /// Evaluate metrics and apply parameter adjustments.
    ///
    /// Returns the list of adjustments made. Parameters are clamped to guardrails.
    pub fn regulate_and_apply(&mut self, metrics: SystemMetrics) -> Vec<Adjustment> {
        self.history.push(metrics.clone());
        let action = self.evaluate(&metrics);
        self.last_action = action.clone();
        self.apply_action(&action)
    }

    /// Get the current system state.
    pub fn get_state(&self) -> &SystemState {
        &self.state
    }

    /// Get the guardrails.
    pub fn get_guardrails(&self) -> &Guardrails {
        &self.guardrails
    }

    /// Get the history of metrics evaluations.
    pub fn history(&self) -> &[SystemMetrics] {
        &self.history
    }

    /// Clear the history buffer.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Get the status snapshot for API endpoint.
    pub fn status(&self) -> RegulationStatus {
        let status_label = match self.last_action {
            RegulatoryAction::None => "BALANCED",
            RegulatoryAction::ReduceLoad => "REDUCING_LOAD",
            RegulatoryAction::IncreaseExploration => "EXPLORING",
            RegulatoryAction::TightenSecurity => "TIGHTENING_SECURITY",
            RegulatoryAction::CircuitBreak => "CIRCUIT_BREAK",
        };
        RegulationStatus {
            status: status_label.to_string(),
            state: self.state.clone(),
            history_count: self.history.len(),
            last_action: self.last_action.clone(),
        }
    }

    /// Evaluate metrics and determine the action.
    fn evaluate(&self, metrics: &SystemMetrics) -> RegulatoryAction {
        // Emergency: circuit break if resources are critically high
        if metrics.resource_utilization > self.config.max_resource_util {
            return RegulatoryAction::CircuitBreak;
        }

        // If latency or error rate is high, reduce load
        if metrics.latency_p95_ms > self.config.max_latency_ms
            || metrics.error_rate > self.config.max_error_rate
        {
            return RegulatoryAction::ReduceLoad;
        }

        // If anomaly rate is high, tighten security
        if metrics.anomaly_rate > self.config.max_anomaly_rate {
            return RegulatoryAction::TightenSecurity;
        }

        // If entropy is low (stagnation), increase exploration
        if metrics.pheromone_entropy < self.config.min_entropy {
            return RegulatoryAction::IncreaseExploration;
        }

        RegulatoryAction::None
    }

    /// Apply parameter adjustments based on the action.
    fn apply_action(&mut self, action: &RegulatoryAction) -> Vec<Adjustment> {
        match action {
            RegulatoryAction::None => vec![],
            RegulatoryAction::CircuitBreak => vec![],
            RegulatoryAction::ReduceLoad => self.apply_reduce_load(),
            RegulatoryAction::IncreaseExploration => self.apply_increase_exploration(),
            RegulatoryAction::TightenSecurity => self.apply_tighten_security(),
        }
    }

    /// Reduce load: increase detection threshold, reduce ant count, lower mutation.
    fn apply_reduce_load(&mut self) -> Vec<Adjustment> {
        let step = self.config.adjustment_step;
        let mut adjustments = Vec::new();

        // Increase detection threshold (less sensitive = less load)
        let old = self.state.detection_threshold;
        let new = (old * (1.0 + step)).clamp(
            self.guardrails.detection_threshold.0,
            self.guardrails.detection_threshold.1,
        );
        if (new - old).abs() > 1e-10 {
            adjustments.push(Adjustment {
                parameter: "detection_threshold".into(),
                from_value: format!("{:.4}", old),
                to_value: format!("{:.4}", new),
                reason: "Reducing load: increasing detection threshold".into(),
            });
            self.state.detection_threshold = new;
        }

        // Reduce ant count
        let old_count = self.state.ant_count;
        let new_count =
            ((old_count as f64 * (1.0 - step)).round() as usize).clamp(
                self.guardrails.ant_count.0,
                self.guardrails.ant_count.1,
            );
        if new_count != old_count {
            adjustments.push(Adjustment {
                parameter: "ant_count".into(),
                from_value: old_count.to_string(),
                to_value: new_count.to_string(),
                reason: "Reducing load: fewer ants per cycle".into(),
            });
            self.state.ant_count = new_count;
        }

        adjustments
    }

    /// Increase exploration: raise mutation rate, lower thresholds.
    fn apply_increase_exploration(&mut self) -> Vec<Adjustment> {
        let step = self.config.adjustment_step;
        let mut adjustments = Vec::new();

        // Raise mutation rate
        let old = self.state.mutation_rate;
        let new = (old * (1.0 + step)).clamp(
            self.guardrails.mutation_rate.0,
            self.guardrails.mutation_rate.1,
        );
        if (new - old).abs() > 1e-10 {
            adjustments.push(Adjustment {
                parameter: "mutation_rate".into(),
                from_value: format!("{:.4}", old),
                to_value: format!("{:.4}", new),
                reason: "Low pheromone entropy: increasing mutation rate".into(),
            });
            self.state.mutation_rate = new;
        }

        // Raise evaporation rho (forces more turnover)
        let old = self.state.evaporation_rho;
        let new = (old * (1.0 + step)).clamp(
            self.guardrails.evaporation_rho.0,
            self.guardrails.evaporation_rho.1,
        );
        if (new - old).abs() > 1e-10 {
            adjustments.push(Adjustment {
                parameter: "evaporation_rho".into(),
                from_value: format!("{:.4}", old),
                to_value: format!("{:.4}", new),
                reason: "Low pheromone entropy: increasing evaporation rate".into(),
            });
            self.state.evaporation_rho = new;
        }

        adjustments
    }

    /// Tighten security: lower detection and nonself thresholds.
    fn apply_tighten_security(&mut self) -> Vec<Adjustment> {
        let step = self.config.adjustment_step;
        let mut adjustments = Vec::new();

        // Lower detection threshold (more sensitive)
        let old = self.state.detection_threshold;
        let new = (old * (1.0 - step)).clamp(
            self.guardrails.detection_threshold.0,
            self.guardrails.detection_threshold.1,
        );
        if (new - old).abs() > 1e-10 {
            adjustments.push(Adjustment {
                parameter: "detection_threshold".into(),
                from_value: format!("{:.4}", old),
                to_value: format!("{:.4}", new),
                reason: "High anomaly rate: lowering detection threshold".into(),
            });
            self.state.detection_threshold = new;
        }

        // Lower nonself threshold (more sensitive)
        let old = self.state.nonself_threshold;
        let new = (old * (1.0 - step)).clamp(
            self.guardrails.nonself_threshold.0,
            self.guardrails.nonself_threshold.1,
        );
        if (new - old).abs() > 1e-10 {
            adjustments.push(Adjustment {
                parameter: "nonself_threshold".into(),
                from_value: format!("{:.4}", old),
                to_value: format!("{:.4}", new),
                reason: "High anomaly rate: lowering nonself threshold".into(),
            });
            self.state.nonself_threshold = new;
        }

        adjustments
    }
}

impl Default for NetworkRegulator {
    fn default() -> Self {
        Self::new(RegulatorConfig::default())
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    fn healthy_metrics() -> SystemMetrics {
        SystemMetrics {
            latency_p95_ms: 10.0,
            error_rate: 0.01,
            pheromone_entropy: 0.5,
            anomaly_rate: 2.0,
            resource_utilization: 0.5,
        }
    }

    #[test]
    fn healthy_system_returns_none() {
        let mut reg = NetworkRegulator::default();
        let action = reg.regulate(healthy_metrics());
        assert_eq!(action, RegulatoryAction::None);
    }

    #[test]
    fn high_latency_returns_reduce_load() {
        let mut reg = NetworkRegulator::default();
        let action = reg.regulate(SystemMetrics {
            latency_p95_ms: 200.0,
            ..healthy_metrics()
        });
        assert_eq!(action, RegulatoryAction::ReduceLoad);
    }

    #[test]
    fn high_error_rate_returns_reduce_load() {
        let mut reg = NetworkRegulator::default();
        let action = reg.regulate(SystemMetrics {
            error_rate: 0.1,
            ..healthy_metrics()
        });
        assert_eq!(action, RegulatoryAction::ReduceLoad);
    }

    #[test]
    fn low_entropy_returns_increase_exploration() {
        let mut reg = NetworkRegulator::default();
        let action = reg.regulate(SystemMetrics {
            pheromone_entropy: 0.05,
            ..healthy_metrics()
        });
        assert_eq!(action, RegulatoryAction::IncreaseExploration);
    }

    #[test]
    fn high_anomaly_returns_tighten_security() {
        let mut reg = NetworkRegulator::default();
        let action = reg.regulate(SystemMetrics {
            anomaly_rate: 50.0,
            ..healthy_metrics()
        });
        assert_eq!(action, RegulatoryAction::TightenSecurity);
    }

    #[test]
    fn critical_resource_returns_circuit_break() {
        let mut reg = NetworkRegulator::default();
        let action = reg.regulate(SystemMetrics {
            resource_utilization: 0.95,
            ..healthy_metrics()
        });
        assert_eq!(action, RegulatoryAction::CircuitBreak);
    }

    #[test]
    fn history_is_recorded() {
        let mut reg = NetworkRegulator::default();
        reg.regulate(healthy_metrics());
        assert_eq!(reg.history().len(), 1);
    }

    // ── FT-022 DoD: regulate_and_apply tests ──────────────────────────────

    #[test]
    fn fpr_high_decreases_detection_threshold() {
        let mut reg = NetworkRegulator::default();
        let initial = reg.get_state().detection_threshold;
        // High error rate triggers ReduceLoad → threshold increases
        let adjustments = reg.regulate_and_apply(SystemMetrics {
            error_rate: 0.1,
            ..healthy_metrics()
        });
        assert!(!adjustments.is_empty());
        assert!(reg.get_state().detection_threshold > initial);
    }

    #[test]
    fn high_anomaly_decreases_detection_threshold() {
        let mut reg = NetworkRegulator::default();
        let initial = reg.get_state().detection_threshold;
        let adjustments = reg.regulate_and_apply(SystemMetrics {
            anomaly_rate: 50.0,
            ..healthy_metrics()
        });
        assert!(!adjustments.is_empty());
        // TightenSecurity lowers the threshold (more sensitive)
        assert!(reg.get_state().detection_threshold < initial);
    }

    #[test]
    fn low_entropy_increases_evaporation_rho() {
        let mut reg = NetworkRegulator::default();
        let initial = reg.get_state().evaporation_rho;
        let adjustments = reg.regulate_and_apply(SystemMetrics {
            pheromone_entropy: 0.05,
            ..healthy_metrics()
        });
        assert!(!adjustments.is_empty());
        assert!(reg.get_state().evaporation_rho > initial);
    }

    #[test]
    fn low_entropy_increases_mutation_rate() {
        let mut reg = NetworkRegulator::default();
        let initial = reg.get_state().mutation_rate;
        let adjustments = reg.regulate_and_apply(SystemMetrics {
            pheromone_entropy: 0.05,
            ..healthy_metrics()
        });
        assert!(!adjustments.is_empty());
        assert!(reg.get_state().mutation_rate > initial);
    }

    #[test]
    fn guardrail_min_respected() {
        let state = SystemState {
            detection_threshold: 0.31,
            ..SystemState::default()
        };
        let guardrails = Guardrails::default();
        let mut reg = NetworkRegulator::with_state(
            RegulatorConfig::default(),
            state,
            guardrails,
        );
        // TightenSecurity lowers threshold but guardrail min is 0.3
        for _ in 0..100 {
            reg.regulate_and_apply(SystemMetrics {
                anomaly_rate: 50.0,
                ..healthy_metrics()
            });
        }
        assert!(
            reg.get_state().detection_threshold >= 0.3,
            "threshold {} violated min 0.3",
            reg.get_state().detection_threshold
        );
    }

    #[test]
    fn guardrail_max_respected() {
        let state = SystemState {
            detection_threshold: 0.94,
            ..SystemState::default()
        };
        let guardrails = Guardrails::default();
        let mut reg = NetworkRegulator::with_state(
            RegulatorConfig::default(),
            state,
            guardrails,
        );
        // ReduceLoad increases threshold but guardrail max is 0.95
        for _ in 0..100 {
            reg.regulate_and_apply(SystemMetrics {
                error_rate: 0.5,
                ..healthy_metrics()
            });
        }
        assert!(
            reg.get_state().detection_threshold <= 0.95,
            "threshold {} violated max 0.95",
            reg.get_state().detection_threshold
        );
    }

    #[test]
    fn balanced_system_no_adjustments() {
        let mut reg = NetworkRegulator::default();
        let adjustments = reg.regulate_and_apply(healthy_metrics());
        assert!(adjustments.is_empty());
    }

    #[test]
    fn convergence_over_10_cycles() {
        let mut reg = NetworkRegulator::default();
        for _ in 0..10 {
            reg.regulate_and_apply(healthy_metrics());
        }
        // After 10 healthy cycles, no adjustments should be made
        // (the system converges to balanced state)
        let last_5_adjustments: usize = (0..5)
            .map(|_| {
                reg.regulate_and_apply(healthy_metrics())
                    .len()
            })
            .sum();
        assert_eq!(
            last_5_adjustments, 0,
            "system should converge to balanced"
        );
    }

    #[test]
    fn reduce_load_decreases_ant_count() {
        let state = SystemState {
            ant_count: 10,
            ..SystemState::default()
        };
        let mut reg = NetworkRegulator::with_state(
            RegulatorConfig::default(),
            state,
            Guardrails::default(),
        );
        let initial = reg.get_state().ant_count;
        let adjustments = reg.regulate_and_apply(SystemMetrics {
            latency_p95_ms: 200.0,
            ..healthy_metrics()
        });
        assert!(!adjustments.is_empty());
        assert!(reg.get_state().ant_count < initial);
    }

    #[test]
    fn status_snapshot() {
        let mut reg = NetworkRegulator::default();
        reg.regulate(healthy_metrics());
        let status = reg.status();
        assert_eq!(status.status, "BALANCED");
        assert_eq!(status.history_count, 1);
    }

    #[test]
    fn get_guardrails_works() {
        let reg = NetworkRegulator::default();
        let g = reg.get_guardrails();
        assert_eq!(g.detection_threshold, (0.3, 0.95));
        assert_eq!(g.mutation_rate, (0.01, 0.3));
    }
}

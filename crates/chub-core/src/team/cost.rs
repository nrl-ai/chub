//! Token-to-cost estimation with built-in rates for known models.
//!
//! Custom rates from `.chub/config.yaml` `tracking.cost_rates` take priority
//! over built-in rates.

use super::sessions::TokenUsage;
use crate::config::{self, CustomCostRate};

/// Per-million-token pricing.
struct ModelRate {
    input_per_m: f64,
    output_per_m: f64,
    cache_read_per_m: f64,
    cache_write_per_m: f64,
    /// Reasoning/thinking token rate. Defaults to output rate if not set.
    reasoning_per_m: f64,
}

impl From<&CustomCostRate> for ModelRate {
    fn from(r: &CustomCostRate) -> Self {
        Self {
            input_per_m: r.input_per_m,
            output_per_m: r.output_per_m,
            cache_read_per_m: r.cache_read_per_m.unwrap_or(r.input_per_m * 0.1),
            cache_write_per_m: r.cache_write_per_m.unwrap_or(r.input_per_m * 1.25),
            reasoning_per_m: r.output_per_m, // default: same as output
        }
    }
}

/// Estimate cost in USD for a given model and token usage.
/// Checks user-configured rates first, then falls back to built-in rates.
pub fn estimate_cost(model: Option<&str>, tokens: &TokenUsage) -> Option<f64> {
    let rate = model.and_then(|m| custom_rate(m).or_else(|| model_rate(m)))?;

    let cost = (tokens.input as f64 * rate.input_per_m
        + tokens.output as f64 * rate.output_per_m
        + tokens.cache_read as f64 * rate.cache_read_per_m
        + tokens.cache_write as f64 * rate.cache_write_per_m
        + tokens.reasoning as f64 * rate.reasoning_per_m)
        / 1_000_000.0;

    Some((cost * 1000.0).round() / 1000.0) // Round to 3 decimal places
}

/// Check user-configured cost rates from `.chub/config.yaml`.
fn custom_rate(model: &str) -> Option<ModelRate> {
    let cfg = config::load_config();
    let m = model.to_lowercase();
    cfg.tracking
        .cost_rates
        .iter()
        .find(|r| m.contains(&r.model.to_lowercase()))
        .map(ModelRate::from)
}

fn model_rate(model: &str) -> Option<ModelRate> {
    let m = model.to_lowercase();

    // Claude models — reasoning tokens priced at output rate
    if m.contains("opus") {
        return Some(ModelRate {
            input_per_m: 15.0,
            output_per_m: 75.0,
            cache_read_per_m: 1.5,
            cache_write_per_m: 18.75,
            reasoning_per_m: 75.0,
        });
    }
    if m.contains("sonnet") {
        return Some(ModelRate {
            input_per_m: 3.0,
            output_per_m: 15.0,
            cache_read_per_m: 0.3,
            cache_write_per_m: 3.75,
            reasoning_per_m: 15.0,
        });
    }
    if m.contains("haiku") {
        return Some(ModelRate {
            input_per_m: 0.80,
            output_per_m: 4.0,
            cache_read_per_m: 0.08,
            cache_write_per_m: 1.0,
            reasoning_per_m: 4.0,
        });
    }

    // GPT models
    if m.contains("gpt-4o") && !m.contains("mini") {
        return Some(ModelRate {
            input_per_m: 2.50,
            output_per_m: 10.0,
            cache_read_per_m: 1.25,
            cache_write_per_m: 2.50,
            reasoning_per_m: 10.0,
        });
    }
    if m.contains("gpt-4o-mini") {
        return Some(ModelRate {
            input_per_m: 0.15,
            output_per_m: 0.60,
            cache_read_per_m: 0.075,
            cache_write_per_m: 0.15,
            reasoning_per_m: 0.60,
        });
    }
    if m.contains("o3") || m.contains("o1") {
        return Some(ModelRate {
            input_per_m: 10.0,
            output_per_m: 40.0,
            cache_read_per_m: 2.5,
            cache_write_per_m: 10.0,
            reasoning_per_m: 40.0,
        });
    }

    // Gemini models — thinking tokens priced at output rate
    if m.contains("gemini") && m.contains("pro") {
        return Some(ModelRate {
            input_per_m: 1.25,
            output_per_m: 5.0,
            cache_read_per_m: 0.315,
            cache_write_per_m: 1.25,
            reasoning_per_m: 5.0,
        });
    }
    if m.contains("gemini") && m.contains("flash") {
        return Some(ModelRate {
            input_per_m: 0.075,
            output_per_m: 0.30,
            cache_read_per_m: 0.01875,
            cache_write_per_m: 0.075,
            reasoning_per_m: 0.30,
        });
    }

    // DeepSeek — reasoning tokens at output rate
    if m.contains("deepseek") {
        return Some(ModelRate {
            input_per_m: 0.27,
            output_per_m: 1.10,
            cache_read_per_m: 0.07,
            cache_write_per_m: 0.27,
            reasoning_per_m: 1.10,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_opus_cost() {
        let tokens = TokenUsage {
            input: 10000,
            output: 5000,
            cache_read: 2000,
            cache_write: 1000,
            ..Default::default()
        };
        let cost = estimate_cost(Some("claude-opus-4-6"), &tokens).unwrap();
        // input: 10000 * 15 / 1M = 0.15
        // output: 5000 * 75 / 1M = 0.375
        // cache_read: 2000 * 1.5 / 1M = 0.003
        // cache_write: 1000 * 18.75 / 1M = 0.01875
        // total ≈ 0.547
        assert!(cost > 0.5 && cost < 0.6, "cost was {}", cost);
    }

    #[test]
    fn unknown_model_returns_none() {
        let tokens = TokenUsage {
            input: 1000,
            output: 500,
            ..Default::default()
        };
        assert!(estimate_cost(Some("unknown-model-xyz"), &tokens).is_none());
        assert!(estimate_cost(None, &tokens).is_none());
    }

    #[test]
    fn sonnet_cost() {
        let tokens = TokenUsage {
            input: 100000,
            output: 20000,
            ..Default::default()
        };
        let cost = estimate_cost(Some("claude-sonnet-4-6"), &tokens).unwrap();
        // input: 100000 * 3 / 1M = 0.3
        // output: 20000 * 15 / 1M = 0.3
        // total = 0.6
        assert!((cost - 0.6).abs() < 0.01, "cost was {}", cost);
    }
}

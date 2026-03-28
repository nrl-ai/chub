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

    #[test]
    fn haiku_cost() {
        let tokens = TokenUsage {
            input: 50000,
            output: 10000,
            ..Default::default()
        };
        let cost = estimate_cost(Some("claude-haiku-4-5"), &tokens).unwrap();
        // input: 50000 * 0.80 / 1M = 0.04
        // output: 10000 * 4.0 / 1M = 0.04
        assert!((cost - 0.08).abs() < 0.01, "haiku cost was {}", cost);
    }

    #[test]
    fn gpt4o_cost() {
        let tokens = TokenUsage {
            input: 10000,
            output: 5000,
            ..Default::default()
        };
        let cost = estimate_cost(Some("gpt-4o-2024-08-06"), &tokens).unwrap();
        // input: 10000 * 2.5 / 1M = 0.025
        // output: 5000 * 10 / 1M = 0.05
        assert!(cost > 0.07 && cost < 0.08, "gpt-4o cost was {}", cost);
    }

    #[test]
    fn gpt4o_mini_cost() {
        let tokens = TokenUsage {
            input: 100000,
            output: 50000,
            ..Default::default()
        };
        let cost = estimate_cost(Some("gpt-4o-mini"), &tokens).unwrap();
        // input: 100000 * 0.15 / 1M = 0.015
        // output: 50000 * 0.60 / 1M = 0.03
        assert!(cost > 0.04 && cost < 0.05, "gpt-4o-mini cost was {}", cost);
    }

    #[test]
    fn o3_cost() {
        let tokens = TokenUsage {
            input: 10000,
            output: 5000,
            reasoning: 20000,
            ..Default::default()
        };
        let cost = estimate_cost(Some("o3-2025-01-31"), &tokens).unwrap();
        // input: 10000 * 10 / 1M = 0.1
        // output: 5000 * 40 / 1M = 0.2
        // reasoning: 20000 * 40 / 1M = 0.8
        assert!((cost - 1.1).abs() < 0.01, "o3 cost was {}", cost);
    }

    #[test]
    fn gemini_pro_cost() {
        let tokens = TokenUsage {
            input: 50000,
            output: 10000,
            ..Default::default()
        };
        let cost = estimate_cost(Some("gemini-2.5-pro"), &tokens).unwrap();
        assert!(cost > 0.0, "gemini pro should have a cost");
    }

    #[test]
    fn gemini_flash_cost() {
        let tokens = TokenUsage {
            input: 100000,
            output: 20000,
            ..Default::default()
        };
        let cost = estimate_cost(Some("gemini-2.0-flash"), &tokens).unwrap();
        assert!(cost > 0.0, "gemini flash should have a cost");
        // Flash should be cheaper than pro
        let pro_cost = estimate_cost(Some("gemini-2.5-pro"), &tokens).unwrap();
        assert!(
            cost < pro_cost,
            "flash ({}) should be cheaper than pro ({})",
            cost,
            pro_cost
        );
    }

    #[test]
    fn deepseek_cost() {
        let tokens = TokenUsage {
            input: 100000,
            output: 50000,
            ..Default::default()
        };
        let cost = estimate_cost(Some("deepseek-coder"), &tokens).unwrap();
        assert!(cost > 0.0, "deepseek should have a cost");
    }

    #[test]
    fn reasoning_tokens_priced() {
        let tokens = TokenUsage {
            reasoning: 10000,
            ..Default::default()
        };
        let cost = estimate_cost(Some("claude-opus-4-6"), &tokens).unwrap();
        // reasoning: 10000 * 75 / 1M = 0.75
        assert!((cost - 0.75).abs() < 0.01, "reasoning cost was {}", cost);
    }

    #[test]
    fn cache_tokens_priced_correctly() {
        let tokens = TokenUsage {
            cache_read: 1000000,
            ..Default::default()
        };
        let cost = estimate_cost(Some("claude-opus-4-6"), &tokens).unwrap();
        // cache_read: 1M * 1.5 / 1M = 1.5
        assert!((cost - 1.5).abs() < 0.01, "cache read cost was {}", cost);

        let tokens2 = TokenUsage {
            cache_write: 1000000,
            ..Default::default()
        };
        let cost2 = estimate_cost(Some("claude-opus-4-6"), &tokens2).unwrap();
        // cache_write: 1M * 18.75 / 1M = 18.75
        assert!(
            (cost2 - 18.75).abs() < 0.01,
            "cache write cost was {}",
            cost2
        );
    }

    #[test]
    fn zero_tokens_zero_cost() {
        let tokens = TokenUsage::default();
        let cost = estimate_cost(Some("claude-opus-4-6"), &tokens).unwrap();
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn model_matching_is_case_insensitive() {
        let tokens = TokenUsage {
            input: 10000,
            output: 5000,
            ..Default::default()
        };
        let lower = estimate_cost(Some("claude-opus-4-6"), &tokens).unwrap();
        let upper = estimate_cost(Some("Claude-OPUS-4-6"), &tokens).unwrap();
        assert_eq!(lower, upper, "model matching should be case-insensitive");
    }

    #[test]
    fn cost_rounded_to_three_decimals() {
        // A cost that would have more than 3 decimal places
        let tokens = TokenUsage {
            input: 1,
            output: 1,
            ..Default::default()
        };
        let cost = estimate_cost(Some("claude-opus-4-6"), &tokens).unwrap();
        // Very small: (1 * 15 + 1 * 75) / 1_000_000 = 0.00009 → rounds to 0.0
        let s = format!("{}", cost);
        let decimals = s.split('.').nth(1).unwrap_or("").len();
        assert!(decimals <= 3, "cost should be rounded to 3 decimals: {}", s);
    }
}

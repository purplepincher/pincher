//! Ternary logic adapter for the veto system.
//!
//! Bridges [`crate::route::Ternary`] to [`VetoDecision`] with Kleene logic
//! operations and confidence-threshold bridging.
//!
//! # Kleene Logic
//!
//! Ternary values form a three-valued (Kleene) logic where:
//!
//! | a | b | a ∧ b | a ∨ b | ¬a |
//! |---|---|---|---|---|
//! | Positive | Positive | Positive | Positive | Negative |
//! | Positive | Neutral | Neutral | Positive | Negative |
//! | Positive | Negative | Negative | Positive | Negative |
//! | Neutral | Neutral | Neutral | Neutral | Neutral |
//! | Neutral | Negative | Negative | Neutral | Neutral |
//! | Negative | Negative | Negative | Negative | Positive |
//!
//! This matches the veto system's three decision states.

use crate::route::Ternary;
use crate::security::VetoDecision;

/// Kleene conjunction (AND) on ternary values.
pub fn kleene_and(a: Ternary, b: Ternary) -> Ternary {
    use Ternary::{Negative, Neutral, Positive};
    match (a, b) {
        (Negative, _) | (_, Negative) => Negative,
        (Neutral, _) | (_, Neutral) => Neutral,
        (Positive, Positive) => Positive,
    }
}

/// Kleene disjunction (OR) on ternary values.
pub fn kleene_or(a: Ternary, b: Ternary) -> Ternary {
    use Ternary::{Negative, Neutral, Positive};
    match (a, b) {
        (Positive, _) | (_, Positive) => Positive,
        (Neutral, _) | (_, Neutral) => Neutral,
        (Negative, Negative) => Negative,
    }
}

/// Kleene negation (NOT) on a ternary value.
pub fn kleene_not(a: Ternary) -> Ternary {
    use Ternary::{Negative, Neutral, Positive};
    match a {
        Positive => Negative,
        Neutral => Neutral,
        Negative => Positive,
    }
}

/// Convert a confidence value in `[0.0, 1.0]` to a [`Ternary`] decision.
///
/// - `confidence >= high_threshold` → `Positive`
/// - `confidence <= low_threshold` → `Negative`
/// - otherwise → `Neutral`
pub fn from_confidence(confidence: f64, high_threshold: f64, low_threshold: f64) -> Ternary {
    if confidence >= high_threshold {
        Ternary::Positive
    } else if confidence <= low_threshold {
        Ternary::Negative
    } else {
        Ternary::Neutral
    }
}

/// Convert a [`Ternary`] to a [`VetoDecision`].
///
/// - `Positive` → `Allow`
/// - `Neutral` → `RequireConfirmation(reason)` with the given reason
/// - `Negative` → `Deny(reason)` with the given reason
pub fn ternary_to_veto(value: Ternary, reason: impl Into<String>) -> VetoDecision {
    let reason = reason.into();
    match value {
        Ternary::Positive => VetoDecision::Allow,
        Ternary::Neutral => VetoDecision::RequireConfirmation(reason),
        Ternary::Negative => VetoDecision::Deny(reason),
    }
}

/// Convert a [`VetoDecision`] to a [`Ternary`].
///
/// - `Allow` → `Positive`
/// - `RequireConfirmation(_)` → `Neutral`
/// - `Deny(_)` → `Negative`
pub fn veto_to_ternary(decision: &VetoDecision) -> Ternary {
    match decision {
        VetoDecision::Allow => Ternary::Positive,
        VetoDecision::RequireConfirmation(_) => Ternary::Neutral,
        VetoDecision::Deny(_) => Ternary::Negative,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Ternary::{Negative, Neutral, Positive};

    #[test]
    fn test_kleene_and() {
        assert_eq!(kleene_and(Positive, Positive), Positive);
        assert_eq!(kleene_and(Positive, Neutral), Neutral);
        assert_eq!(kleene_and(Positive, Negative), Negative);
        assert_eq!(kleene_and(Neutral, Negative), Negative);
        assert_eq!(kleene_and(Neutral, Neutral), Neutral);
        assert_eq!(kleene_and(Negative, Negative), Negative);
    }

    #[test]
    fn test_kleene_or() {
        assert_eq!(kleene_or(Positive, Positive), Positive);
        assert_eq!(kleene_or(Positive, Neutral), Positive);
        assert_eq!(kleene_or(Positive, Negative), Positive);
        assert_eq!(kleene_or(Neutral, Negative), Neutral);
        assert_eq!(kleene_or(Neutral, Neutral), Neutral);
        assert_eq!(kleene_or(Negative, Negative), Negative);
    }

    #[test]
    fn test_kleene_not() {
        assert_eq!(kleene_not(Positive), Negative);
        assert_eq!(kleene_not(Negative), Positive);
        assert_eq!(kleene_not(Neutral), Neutral);
    }

    #[test]
    fn test_from_confidence() {
        assert_eq!(from_confidence(0.9, 0.7, 0.3), Positive);
        assert_eq!(from_confidence(0.1, 0.7, 0.3), Negative);
        assert_eq!(from_confidence(0.5, 0.7, 0.3), Neutral);
    }

    #[test]
    fn test_ternary_to_veto_allow() {
        let v = ternary_to_veto(Positive, "should not appear");
        assert_eq!(v, VetoDecision::Allow);
    }

    #[test]
    fn test_ternary_to_veto_require_confirmation() {
        let v = ternary_to_veto(Neutral, "needs review");
        assert_eq!(v, VetoDecision::RequireConfirmation("needs review".into()));
    }

    #[test]
    fn test_ternary_to_veto_deny() {
        let v = ternary_to_veto(Negative, "blocked");
        assert_eq!(v, VetoDecision::Deny("blocked".into()));
    }

    #[test]
    fn test_veto_to_ternary_roundtrip() {
        let decisions = vec![
            VetoDecision::Allow,
            VetoDecision::RequireConfirmation("test".into()),
            VetoDecision::Deny("test".into()),
        ];
        for d in &decisions {
            let t = veto_to_ternary(d);
            let back = ternary_to_veto(t, "test");
            assert_eq!(d.clone(), back, "roundtrip failed for {d:?}");
        }
    }

    #[test]
    fn test_kleene_associativity() {
        // (a ∧ b) ∧ c == a ∧ (b ∧ c)
        let values = [Positive, Neutral, Negative];
        for &a in &values {
            for &b in &values {
                for &c in &values {
                    let left = kleene_and(kleene_and(a, b), c);
                    let right = kleene_and(a, kleene_and(b, c));
                    assert_eq!(left, right, "AND not associative for {a:?} {b:?} {c:?}");
                }
            }
        }
    }

    #[test]
    fn test_kleene_de_morgan() {
        // ¬(a ∨ b) == ¬a ∧ ¬b
        let values = [Positive, Neutral, Negative];
        for &a in &values {
            for &b in &values {
                let left = kleene_not(kleene_or(a, b));
                let right = kleene_and(kleene_not(a), kleene_not(b));
                assert_eq!(left, right, "De Morgan failed for {a:?} {b:?}");
            }
        }
    }

    #[test]
    fn test_kleene_commutativity() {
        let values = [Positive, Neutral, Negative];
        for &a in &values {
            for &b in &values {
                assert_eq!(kleene_and(a, b), kleene_and(b, a), "AND not commutative");
                assert_eq!(kleene_or(a, b), kleene_or(b, a), "OR not commutative");
            }
        }
    }

    #[test]
    fn test_veto_to_ternary_direct() {
        assert_eq!(veto_to_ternary(&VetoDecision::Allow), Positive);
        assert_eq!(
            veto_to_ternary(&VetoDecision::RequireConfirmation("r".into())),
            Neutral
        );
        assert_eq!(veto_to_ternary(&VetoDecision::Deny("r".into())), Negative);
    }
}

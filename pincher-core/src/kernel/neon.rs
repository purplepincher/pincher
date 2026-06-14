//! # ARM NEON SIMD Kernels
//!
//! f32 dot product, L2 normalization, and vector scaling via NEON
//! intrinsics. Only compiled on `aarch64` with the `ternary-kernel`
//! feature.
//!
//! ## Safety
//!
//! These functions use `unsafe` for NEON intrinsics. They are safe to
//! call from the public API — the unsafe is contained here with
//! pre-flight length checks.

use core::arch::aarch64::*;

/// NEON-accelerated f32 dot product.
///
/// Processes 4 f32 lanes per iteration with `vaddq_f32` accumulation,
/// then horizontally sums the tail elements.
///
/// # Panics
///
/// Panics if `a.len() != b.len()`.
#[allow(dead_code)]
pub(crate) fn neon_dot_product(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "a and b must have equal length");
    let n = a.len();
    let mut i = 0;

    unsafe {
        // Accumulate in two vector lanes to reduce serial dependency
        let mut sum0 = vdupq_n_f32(0.0);
        let mut sum1 = vdupq_n_f32(0.0);

        // Process 8 elements per iteration (2 × 4-lane vectors)
        while i + 8 <= n {
            let a0 = vld1q_f32(a.as_ptr().add(i));
            let b0 = vld1q_f32(b.as_ptr().add(i));
            sum0 = vmlaq_f32(sum0, a0, b0);

            let a1 = vld1q_f32(a.as_ptr().add(i + 4));
            let b1 = vld1q_f32(b.as_ptr().add(i + 4));
            sum1 = vmlaq_f32(sum1, a1, b1);

            i += 8;
        }

        // Handle remaining elements (4 at a time)
        while i + 4 <= n {
            let a0 = vld1q_f32(a.as_ptr().add(i));
            let b0 = vld1q_f32(b.as_ptr().add(i));
            sum0 = vmlaq_f32(sum0, a0, b0);
            i += 4;
        }

        // Reduce to scalar
        let combined = vaddq_f32(sum0, sum1);
        let mut result = vaddvq_f32(combined);

        // Tail elements (0-3 remaining)
        while i < n {
            result += a[i] * b[i];
            i += 1;
        }

        result
    }
}

/// NEON-accelerated squared-sum reduction (for L2 norm computation).
///
/// Computes Σ(vec[i]²) using parallel f32 accumulators.
pub(crate) fn neon_squared_sum(vec: &[f32]) -> f32 {
    let n = vec.len();
    let mut i = 0;

    unsafe {
        let mut sum0 = vdupq_n_f32(0.0);
        let mut sum1 = vdupq_n_f32(0.0);

        while i + 8 <= n {
            let v0 = vld1q_f32(vec.as_ptr().add(i));
            sum0 = vmlaq_f32(sum0, v0, v0);

            let v1 = vld1q_f32(vec.as_ptr().add(i + 4));
            sum1 = vmlaq_f32(sum1, v1, v1);

            i += 8;
        }

        while i + 4 <= n {
            let v0 = vld1q_f32(vec.as_ptr().add(i));
            sum0 = vmlaq_f32(sum0, v0, v0);
            i += 4;
        }

        let combined = vaddq_f32(sum0, sum1);
        let mut result = vaddvq_f32(combined);

        while i < n {
            result += vec[i] * vec[i];
            i += 1;
        }

        result
    }
}

/// NEON-accelerated cosine similarity.
///
/// Computes `dot(a, b) / (||a|| * ||b||)` using SIMD for all three
/// reductions.
pub(crate) fn neon_cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let n = a.len();
    let mut i = 0;

    unsafe {
        let mut dot0 = vdupq_n_f32(0.0);
        let mut dot1 = vdupq_n_f32(0.0);
        let mut na0 = vdupq_n_f32(0.0);
        let mut na1 = vdupq_n_f32(0.0);
        let mut nb0 = vdupq_n_f32(0.0);
        let mut nb1 = vdupq_n_f32(0.0);

        while i + 8 <= n {
            let a0 = vld1q_f32(a.as_ptr().add(i));
            let b0 = vld1q_f32(b.as_ptr().add(i));
            dot0 = vmlaq_f32(dot0, a0, b0);
            na0 = vmlaq_f32(na0, a0, a0);
            nb0 = vmlaq_f32(nb0, b0, b0);

            let a1 = vld1q_f32(a.as_ptr().add(i + 4));
            let b1 = vld1q_f32(b.as_ptr().add(i + 4));
            dot1 = vmlaq_f32(dot1, a1, b1);
            na1 = vmlaq_f32(na1, a1, a1);
            nb1 = vmlaq_f32(nb1, b1, b1);

            i += 8;
        }

        while i + 4 <= n {
            let a0 = vld1q_f32(a.as_ptr().add(i));
            let b0 = vld1q_f32(b.as_ptr().add(i));
            dot0 = vmlaq_f32(dot0, a0, b0);
            na0 = vmlaq_f32(na0, a0, a0);
            nb0 = vmlaq_f32(nb0, b0, b0);
            i += 4;
        }

        let dot = vaddvq_f32(vaddq_f32(dot0, dot1));
        let na = vaddvq_f32(vaddq_f32(na0, na1));
        let nb = vaddvq_f32(vaddq_f32(nb0, nb1));

        let mut dot_s = dot;
        let mut norm_a_s = na;
        let mut norm_b_s = nb;

        while i < n {
            dot_s += a[i] * b[i];
            norm_a_s += a[i] * a[i];
            norm_b_s += b[i] * b[i];
            i += 1;
        }

        let norm_a = norm_a_s.sqrt();
        let norm_b = norm_b_s.sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        dot_s / (norm_a * norm_b)
    }
}

/// NEON-accelerated in-place L2 normalization.
pub(crate) fn neon_l2_normalize(vec: &mut [f32]) {
    let sq = neon_squared_sum(vec);
    let norm = sq.sqrt();
    if norm > 0.0 {
        neon_scale(vec, 1.0 / norm);
    }
}

/// NEON-accelerated in-place vector scaling.
pub(crate) fn neon_scale(vec: &mut [f32], factor: f32) {
    let n = vec.len();
    let mut i = 0;

    unsafe {
        let factor_vec = vdupq_n_f32(factor);

        while i + 8 <= n {
            let v0 = vld1q_f32(vec.as_ptr().add(i));
            let v1 = vld1q_f32(vec.as_ptr().add(i + 4));
            vst1q_f32(vec.as_mut_ptr().add(i), vmulq_f32(v0, factor_vec));
            vst1q_f32(vec.as_mut_ptr().add(i + 4), vmulq_f32(v1, factor_vec));
            i += 8;
        }

        while i + 4 <= n {
            let v0 = vld1q_f32(vec.as_ptr().add(i));
            vst1q_f32(vec.as_mut_ptr().add(i), vmulq_f32(v0, factor_vec));
            i += 4;
        }

        while i < n {
            vec[i] *= factor;
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot_product() {
        let a = [1.0, 2.0, 3.0, 4.0];
        let b = [5.0, 6.0, 7.0, 8.0];
        let dot = neon_dot_product(&a, &b);
        assert!((dot - 70.0).abs() < 1e-5, "Expected 70.0, got {}", dot);
    }

    #[test]
    fn test_dot_product_uneven_length() {
        let a = [1.0, 2.0, 3.0, 4.0, 5.0];
        let b = [5.0, 6.0, 7.0, 8.0, 9.0];
        let dot = neon_dot_product(&a, &b);
        assert!((dot - 115.0).abs() < 1e-5, "Expected 115.0, got {}", dot);
    }

    #[test]
    fn test_squared_sum() {
        let v = [3.0, 4.0];
        let ssq = neon_squared_sum(&v);
        assert!((ssq - 25.0).abs() < 1e-5, "Expected 25.0, got {}", ssq);
    }

    #[test]
    fn test_squared_sum_odd() {
        let v = [1.0, 2.0, 3.0];
        let ssq = neon_squared_sum(&v);
        assert!((ssq - 14.0).abs() < 1e-5, "Expected 14.0, got {}", ssq);
    }

    #[test]
    fn test_neon_cosine_similarity_identical() {
        let a = [3.0, 4.0];
        let sim = neon_cosine_similarity(&a, &a);
        assert!((sim - 1.0).abs() < 1e-6, "Expected 1.0, got {}", sim);
    }

    #[test]
    fn test_neon_cosine_similarity_orthogonal() {
        let a = [1.0, 0.0];
        let b = [0.0, 1.0];
        let sim = neon_cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-6, "Expected 0.0, got {}", sim);
    }

    #[test]
    fn test_neon_cosine_similarity_odd_len() {
        let a = [1.0, 0.0, 0.0];
        let b = [0.0, 1.0, 1.0];
        let sim = neon_cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-6, "Expected 0.0, got {}", sim);
    }

    #[test]
    fn test_neon_scale() {
        let mut v = vec![1.0, 2.0, 3.0, 4.0];
        neon_scale(&mut v, 0.5);
        assert_eq!(v, vec![0.5, 1.0, 1.5, 2.0]);
    }

    #[test]
    fn test_neon_l2_normalize() {
        let mut v = vec![3.0, 4.0];
        neon_l2_normalize(&mut v);
        assert!((v[0] - 0.6).abs() < 1e-6);
        assert!((v[1] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_neon_l2_normalize_zero() {
        let mut v = vec![0.0, 0.0];
        neon_l2_normalize(&mut v);
        assert_eq!(v, vec![0.0, 0.0]);
    }
}

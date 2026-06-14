//! # Ternary Kernel — SIMD-Optimized Compute Kernels
//!
//! Provides architecture-specific SIMD optimizations for ternary-valued
//! operations and embedding computations.
//!
//! ## Feature Gate
//!
//! - `ternary-kernel` (feature): enables SIMD paths. Disabled by default.
//! - On `target_arch = "aarch64"`: uses NEON intrinsics for f32 dot
//!   products, L2 normalization, and vector scaling.
//! - On `target_arch = "x86_64"` (future): would use AVX2/AVX-512.
//!
//! ## Example
//!
//! ```rust,ignore
//! use pincher_core::kernel::fast_cosine_similarity;
//!
//! let a = vec![0.1, 0.2, 0.3, 0.4];
//! let b = vec![0.5, 0.6, 0.7, 0.8];
//! let sim = fast_cosine_similarity(&a, &b);
//! ```

#[cfg(feature = "ternary-kernel")]
mod neon;

/// Compute cosine similarity between two f32 slices.
///
/// Uses SIMD when `ternary-kernel` + target arch permits; falls back to
/// scalar loop otherwise.
pub fn fast_cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    #[cfg(all(feature = "ternary-kernel", target_arch = "aarch64"))]
    {
        return neon::neon_cosine_similarity(a, b);
    }

    #[cfg(not(all(feature = "ternary-kernel", target_arch = "aarch64")))]
    {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        dot / (norm_a * norm_b)
    }
}

/// L2-normalize a mutable f32 slice in-place.
///
/// Uses SIMD when available for the squared-sum reduction.
pub fn fast_l2_normalize(vec: &mut [f32]) {
    #[cfg(all(feature = "ternary-kernel", target_arch = "aarch64"))]
    {
        neon::neon_l2_normalize(vec);
        return;
    }

    #[cfg(not(all(feature = "ternary-kernel", target_arch = "aarch64")))]
    {
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in vec.iter_mut() {
                *x /= norm;
            }
        }
    }
}

/// Scale a mutable f32 slice by a scalar factor.
///
/// Uses SIMD when available.
pub fn fast_scale(vec: &mut [f32], factor: f32) {
    #[cfg(all(feature = "ternary-kernel", target_arch = "aarch64"))]
    {
        neon::neon_scale(vec, factor);
        return;
    }

    #[cfg(not(all(feature = "ternary-kernel", target_arch = "aarch64")))]
    {
        for x in vec.iter_mut() {
            *x *= factor;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let a = [1.0, 2.0, 3.0, 4.0, 5.0];
        let sim = fast_cosine_similarity(&a, &a);
        assert!((sim - 1.0).abs() < 1e-6, "Expected 1.0, got {}", sim);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = [1.0, 0.0];
        let b = [0.0, 1.0];
        let sim = fast_cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-6, "Expected 0.0, got {}", sim);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = [1.0, 2.0];
        let b = [-1.0, -2.0];
        let sim = fast_cosine_similarity(&a, &b);
        assert!((sim - (-1.0)).abs() < 1e-5, "Expected -1.0, got {}", sim);
    }

    #[test]
    fn test_cosine_similarity_mismatched_lengths() {
        let a = [1.0, 2.0];
        let b = [1.0];
        assert_eq!(fast_cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn test_cosine_similarity_empty() {
        let a: [f32; 0] = [];
        let b: [f32; 0] = [];
        assert_eq!(fast_cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn test_l2_normalize() {
        let mut vec = vec![3.0, 4.0];
        fast_l2_normalize(&mut vec);
        assert!((vec[0] - 0.6).abs() < 1e-6);
        assert!((vec[1] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_l2_normalize_zero_vector() {
        let mut vec = vec![0.0, 0.0, 0.0];
        fast_l2_normalize(&mut vec);
        assert_eq!(vec, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_scale() {
        let mut vec = vec![1.0, 2.0, 3.0];
        fast_scale(&mut vec, 2.0);
        assert_eq!(vec, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_scale_zero_factor() {
        let mut vec = vec![1.0, 2.0, 3.0];
        fast_scale(&mut vec, 0.0);
        assert_eq!(vec, vec![0.0, 0.0, 0.0]);
    }
}

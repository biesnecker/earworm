//! Interpolation curves for envelope shaping.
//!
//! Curves define how values transition between two points over time. They are used
//! to shape envelope segments (attack, decay, release) to create more natural or
//! expressive modulation.

/// Interpolation curve types for envelope shaping.
///
/// All curves map a normalized input value [0, 1] to a normalized output value [0, 1],
/// allowing them to be used for any parameter range.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Curve {
    /// Linear interpolation (constant rate of change)
    #[default]
    Linear,

    /// Exponential curve (slow start, fast finish)
    ///
    /// The parameter controls steepness:
    /// - `2.0` = squared curve
    /// - `3.0` = cubed curve
    /// - Higher values create steeper curves
    Exponential(f64),

    /// Logarithmic curve (fast start, slow finish)
    ///
    /// Inverse of exponential. The parameter controls steepness.
    Logarithmic(f64),

    /// Smooth S-curve with ease-in and ease-out
    ///
    /// Uses smoothstep interpolation for gradual acceleration and deceleration.
    SCurve,
}

impl Curve {
    /// Apply the curve to a normalized value.
    ///
    /// # Arguments
    ///
    /// * `t` - Input value, clamped to [0, 1]
    ///
    /// # Returns
    ///
    /// Curved output value in [0, 1]
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::envelopes::Curve;
    ///
    /// let linear = Curve::Linear;
    /// assert_eq!(linear.apply(0.5), 0.5);
    ///
    /// let exp = Curve::Exponential(2.0);
    /// assert_eq!(exp.apply(0.5), 0.25); // 0.5^2
    /// ```
    pub fn apply(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Curve::Linear => t,
            Curve::Exponential(exp) => t.powf(*exp),
            Curve::Logarithmic(exp) => 1.0 - (1.0 - t).powf(*exp),
            Curve::SCurve => {
                // Smoothstep: cubic ease in/out
                t * t * (3.0 - 2.0 * t)
            }
        }
    }

    /// Map a value from one range to another using this curve.
    ///
    /// This is useful for applying curved interpolation between arbitrary parameter values.
    ///
    /// # Arguments
    ///
    /// * `t` - Input value in the `from_range`
    /// * `from_range` - Input range as (min, max)
    /// * `to_range` - Output range as (min, max)
    ///
    /// # Returns
    ///
    /// Mapped value in `to_range` with curve applied
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::envelopes::Curve;
    ///
    /// let curve = Curve::Exponential(2.0);
    /// // Map 0.5 from range [0, 1] to [0, 100] with exponential curve
    /// let result = curve.map(0.5, (0.0, 1.0), (0.0, 100.0));
    /// assert_eq!(result, 25.0); // 0.5^2 * 100 = 25
    /// ```
    pub fn map(&self, t: f64, from_range: (f64, f64), to_range: (f64, f64)) -> f64 {
        let (from_min, from_max) = from_range;
        let (to_min, to_max) = to_range;

        // Normalize to [0, 1]
        let normalized = (t - from_min) / (from_max - from_min);

        // Apply curve
        let curved = self.apply(normalized);

        // Map to target range
        to_min + curved * (to_max - to_min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-10;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn test_linear_curve() {
        let curve = Curve::Linear;
        assert_eq!(curve.apply(0.0), 0.0);
        assert_eq!(curve.apply(0.5), 0.5);
        assert_eq!(curve.apply(1.0), 1.0);
    }

    #[test]
    fn test_exponential_curve() {
        let curve = Curve::Exponential(2.0);
        assert_eq!(curve.apply(0.0), 0.0);
        assert_eq!(curve.apply(0.5), 0.25); // 0.5^2
        assert_eq!(curve.apply(1.0), 1.0);

        let curve = Curve::Exponential(3.0);
        assert_eq!(curve.apply(0.5), 0.125); // 0.5^3
    }

    #[test]
    fn test_logarithmic_curve() {
        let curve = Curve::Logarithmic(2.0);
        assert_eq!(curve.apply(0.0), 0.0);
        assert_eq!(curve.apply(0.5), 0.75); // 1 - 0.5^2
        assert_eq!(curve.apply(1.0), 1.0);
    }

    #[test]
    fn test_scurve() {
        let curve = Curve::SCurve;
        assert_eq!(curve.apply(0.0), 0.0);
        assert_eq!(curve.apply(0.5), 0.5);
        assert_eq!(curve.apply(1.0), 1.0);

        // S-curve should be below linear at 0.25
        assert!(curve.apply(0.25) < 0.25);
        // S-curve should be above linear at 0.75
        assert!(curve.apply(0.75) > 0.75);
    }

    #[test]
    fn test_clamping() {
        let curve = Curve::Linear;
        assert_eq!(curve.apply(-0.5), 0.0);
        assert_eq!(curve.apply(1.5), 1.0);
    }

    #[test]
    fn test_map_basic() {
        let curve = Curve::Linear;
        let result = curve.map(0.5, (0.0, 1.0), (0.0, 100.0));
        assert_eq!(result, 50.0);
    }

    #[test]
    fn test_map_with_exponential() {
        let curve = Curve::Exponential(2.0);
        let result = curve.map(0.5, (0.0, 1.0), (0.0, 100.0));
        assert_eq!(result, 25.0); // 0.5^2 * 100
    }

    #[test]
    fn test_map_different_ranges() {
        let curve = Curve::Linear;
        // Map from [0, 10] to [100, 200]
        let result = curve.map(5.0, (0.0, 10.0), (100.0, 200.0));
        assert_eq!(result, 150.0);
    }

    #[test]
    fn test_map_negative_ranges() {
        let curve = Curve::Linear;
        // Map from [-1, 1] to [0, 1]
        let result = curve.map(0.0, (-1.0, 1.0), (0.0, 1.0));
        assert_eq!(result, 0.5);
    }

    #[test]
    fn test_map_with_logarithmic() {
        let curve = Curve::Logarithmic(2.0);
        let result = curve.map(0.5, (0.0, 1.0), (0.0, 100.0));
        assert_eq!(result, 75.0); // (1 - 0.5^2) * 100
    }

    #[test]
    fn test_default() {
        let curve = Curve::default();
        assert_eq!(curve, Curve::Linear);
    }

    #[test]
    fn test_exponential_symmetry() {
        // Exponential and Logarithmic with same parameter should be inverses
        let exp = Curve::Exponential(2.0);
        let log = Curve::Logarithmic(2.0);

        for t in [0.25, 0.5, 0.75] {
            let exp_result = exp.apply(t);
            let log_result = log.apply(t);
            // exp(t) + log(1-t) should equal 1
            assert!(approx_eq(exp_result + log.apply(1.0 - t), 1.0));
            // log(t) + exp(1-t) should equal 1
            assert!(approx_eq(log_result + exp.apply(1.0 - t), 1.0));
        }
    }
}

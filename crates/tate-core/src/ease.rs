const MIN_EASE: f64 = 1.3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ease(f64);

impl Ease {
    pub fn new(value: f64) -> Self {
        Ease(value.max(MIN_EASE))
    }

    pub fn default_ease() -> Self {
        Ease(2.5)
    }

    pub fn inner(self) -> f64 {
        self.0
    }

    pub fn update(self, quality: f64) -> Self {
        let delta = 0.1 - (5.0 - quality) * (0.08 + (5.0 - quality) * 0.02);
        Ease::new(self.0 + delta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_to_minimum() {
        assert_eq!(Ease::new(0.5).inner(), MIN_EASE);
        assert_eq!(Ease::new(1.0).inner(), MIN_EASE);
        assert_eq!(Ease::new(MIN_EASE).inner(), MIN_EASE);
    }

    #[test]
    fn preserves_valid_values() {
        assert_eq!(Ease::new(2.5).inner(), 2.5);
        assert_eq!(Ease::new(3.0).inner(), 3.0);
    }

    #[test]
    fn clamps_just_below_minimum() {
        assert_eq!(
            Ease::new(MIN_EASE - 0.0001).inner(),
            MIN_EASE,
            "values just below 1.3 must clamp to 1.3"
        );
    }

    #[test]
    fn update_never_goes_below_minimum() {
        let mut e = Ease::new(MIN_EASE);
        for _ in 0..100 {
            e = e.update(1.0);
        }
        assert_eq!(e.inner(), MIN_EASE);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    Again,
    Hard,
    Good,
    Easy,
}

impl Grade {
    pub fn sm2_quality(self) -> f64 {
        match self {
            Grade::Again => 1.0,
            Grade::Hard => 3.0,
            Grade::Good => 4.0,
            Grade::Easy => 5.0,
        }
    }

    pub fn is_lapse(self) -> bool {
        matches!(self, Grade::Again)
    }
}

impl From<Grade> for u8 {
    fn from(g: Grade) -> u8 {
        match g {
            Grade::Again => 1,
            Grade::Hard => 2,
            Grade::Good => 3,
            Grade::Easy => 4,
        }
    }
}

impl TryFrom<u8> for Grade {
    type Error = InvalidGrade;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Grade::Again),
            2 => Ok(Grade::Hard),
            3 => Ok(Grade::Good),
            4 => Ok(Grade::Easy),
            _ => Err(InvalidGrade(value)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidGrade(pub u8);

impl std::fmt::Display for InvalidGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid grade: {} (expected 1-4)", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grade_round_trip() {
        for g in [Grade::Again, Grade::Hard, Grade::Good, Grade::Easy] {
            let n: u8 = g.into();
            let back = Grade::try_from(n).unwrap();
            assert_eq!(g, back);
        }
    }

    #[test]
    fn invalid_grade() {
        assert!(Grade::try_from(0).is_err());
        assert!(Grade::try_from(5).is_err());
    }

    #[test]
    fn sm2_quality_values() {
        assert_eq!(Grade::Again.sm2_quality(), 1.0);
        assert_eq!(Grade::Hard.sm2_quality(), 3.0);
        assert_eq!(Grade::Good.sm2_quality(), 4.0);
        assert_eq!(Grade::Easy.sm2_quality(), 5.0);
    }

    #[test]
    fn lapse_detection() {
        assert!(Grade::Again.is_lapse());
        assert!(!Grade::Hard.is_lapse());
        assert!(!Grade::Good.is_lapse());
        assert!(!Grade::Easy.is_lapse());
    }
}

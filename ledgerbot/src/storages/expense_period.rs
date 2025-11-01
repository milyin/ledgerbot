use std::fmt;

use chrono::{Datelike, NaiveDate};

/// Represents a period (year and month) for expense tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExpensePeriod {
    pub year: i32,
    pub month: u32,
}

impl ExpensePeriod {
    /// Create a new ExpensePeriod
    pub fn new(year: i32, month: u32) -> Result<Self, String> {
        if !(1..=12).contains(&month) {
            return Err(format!(
                "Invalid month: {}. Must be between 1 and 12",
                month
            ));
        }
        Ok(ExpensePeriod { year, month })
    }

    /// Get the current period (based on current date)
    pub fn current() -> Self {
        let now = chrono::Utc::now().naive_utc().date();
        ExpensePeriod {
            year: now.year(),
            month: now.month(),
        }
    }

    /// Create period from a date
    #[allow(dead_code)]
    pub fn from_date(date: NaiveDate) -> Self {
        ExpensePeriod {
            year: date.year(),
            month: date.month(),
        }
    }

    /// Parse period from string in format "YYYY-MM"
    pub fn from_string(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 2 {
            return Err(format!(
                "Invalid period format: '{}'. Expected format: YYYY-MM",
                s
            ));
        }

        let year = parts[0]
            .parse::<i32>()
            .map_err(|_| format!("Invalid year: '{}'", parts[0]))?;
        let month = parts[1]
            .parse::<u32>()
            .map_err(|_| format!("Invalid month: '{}'", parts[1]))?;

        Self::new(year, month)
    }

    /// Get the next period (month)
    #[allow(dead_code)]
    pub fn next_month(&self) -> Self {
        if self.month == 12 {
            ExpensePeriod {
                year: self.year + 1,
                month: 1,
            }
        } else {
            ExpensePeriod {
                year: self.year,
                month: self.month + 1,
            }
        }
    }

    /// Get the previous period (month)
    #[allow(dead_code)]
    pub fn prev_month(&self) -> Self {
        if self.month == 1 {
            ExpensePeriod {
                year: self.year - 1,
                month: 12,
            }
        } else {
            ExpensePeriod {
                year: self.year,
                month: self.month - 1,
            }
        }
    }

    /// Check if a date falls within this period
    #[allow(dead_code)]
    pub fn contains_date(&self, date: NaiveDate) -> bool {
        date.year() == self.year && date.month() == self.month
    }
}

impl fmt::Display for ExpensePeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}", self.year, self.month)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_valid() {
        let period = ExpensePeriod::new(2024, 3).unwrap();
        assert_eq!(period.year, 2024);
        assert_eq!(period.month, 3);
    }

    #[test]
    fn test_new_invalid_month() {
        assert!(ExpensePeriod::new(2024, 0).is_err());
        assert!(ExpensePeriod::new(2024, 13).is_err());
        assert!(ExpensePeriod::new(2024, 100).is_err());
    }

    #[test]
    fn test_from_string_valid() {
        let period = ExpensePeriod::from_string("2024-03").unwrap();
        assert_eq!(period.year, 2024);
        assert_eq!(period.month, 3);

        let period2 = ExpensePeriod::from_string("2024-12").unwrap();
        assert_eq!(period2.year, 2024);
        assert_eq!(period2.month, 12);
    }

    #[test]
    fn test_from_string_invalid() {
        assert!(ExpensePeriod::from_string("2024").is_err());
        assert!(ExpensePeriod::from_string("2024-13").is_err());
        assert!(ExpensePeriod::from_string("invalid").is_err());
        assert!(ExpensePeriod::from_string("2024-00").is_err());
    }

    #[test]
    fn test_to_string() {
        let period = ExpensePeriod::new(2024, 3).unwrap();
        assert_eq!(period.to_string(), "2024-03");

        let period2 = ExpensePeriod::new(2024, 12).unwrap();
        assert_eq!(period2.to_string(), "2024-12");
    }

    #[test]
    fn test_display() {
        let period = ExpensePeriod::new(2024, 3).unwrap();
        assert_eq!(format!("{}", period), "2024-03");
    }

    #[test]
    fn test_from_date() {
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let period = ExpensePeriod::from_date(date);
        assert_eq!(period.year, 2024);
        assert_eq!(period.month, 3);
    }

    #[test]
    fn test_next_month() {
        let period = ExpensePeriod::new(2024, 3).unwrap();
        let next = period.next_month();
        assert_eq!(next.year, 2024);
        assert_eq!(next.month, 4);

        // Test year rollover
        let dec = ExpensePeriod::new(2024, 12).unwrap();
        let next_year = dec.next_month();
        assert_eq!(next_year.year, 2025);
        assert_eq!(next_year.month, 1);
    }

    #[test]
    fn test_prev_month() {
        let period = ExpensePeriod::new(2024, 3).unwrap();
        let prev = period.prev_month();
        assert_eq!(prev.year, 2024);
        assert_eq!(prev.month, 2);

        // Test year rollover
        let jan = ExpensePeriod::new(2024, 1).unwrap();
        let prev_year = jan.prev_month();
        assert_eq!(prev_year.year, 2023);
        assert_eq!(prev_year.month, 12);
    }

    #[test]
    fn test_contains_date() {
        let period = ExpensePeriod::new(2024, 3).unwrap();

        let date_in = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        assert!(period.contains_date(date_in));

        let date_out = NaiveDate::from_ymd_opt(2024, 4, 1).unwrap();
        assert!(!period.contains_date(date_out));

        let date_out2 = NaiveDate::from_ymd_opt(2023, 3, 15).unwrap();
        assert!(!period.contains_date(date_out2));
    }

    #[test]
    fn test_ordering() {
        let p1 = ExpensePeriod::new(2024, 1).unwrap();
        let p2 = ExpensePeriod::new(2024, 2).unwrap();
        let p3 = ExpensePeriod::new(2025, 1).unwrap();

        assert!(p1 < p2);
        assert!(p2 < p3);
        assert!(p1 < p3);
        assert_eq!(p1, p1);
    }
}

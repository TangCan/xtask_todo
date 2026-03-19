//! Recurrence rule for repeating tasks.

use std::fmt;
use std::str::FromStr;

use chrono::Datelike;
use chrono::NaiveDate;

/// Recurrence rule for repeating tasks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RepeatRule {
    Daily,
    Weekly,
    Monthly,
    Yearly,
    /// Weekdays (Mon–Fri).
    Weekdays,
    /// Every n days.
    Custom(u32),
}

impl RepeatRule {
    /// Returns the next due date (YYYY-MM-DD) from a given date string, or None if base is invalid.
    #[must_use]
    pub fn next_due_date(&self, from: &str) -> Option<String> {
        let base = NaiveDate::parse_from_str(from.trim(), "%Y-%m-%d").ok()?;
        let next = match self {
            Self::Daily => base.succ_opt()?,
            Self::Weekly => base + chrono::Duration::days(7),
            Self::Monthly => {
                let (y, m, d) = (base.year(), base.month(), base.day());
                let (next_y, next_m) = if m == 12 { (y + 1, 1u32) } else { (y, m + 1) };
                NaiveDate::from_ymd_opt(next_y, next_m, std::cmp::min(d, 28))?
            }
            Self::Yearly => NaiveDate::from_ymd_opt(base.year() + 1, base.month(), base.day())?,
            Self::Weekdays => {
                let mut d = base.succ_opt()?;
                for _ in 0..7 {
                    if d.weekday() != chrono::Weekday::Sat && d.weekday() != chrono::Weekday::Sun {
                        break;
                    }
                    d = d.succ_opt()?;
                }
                d
            }
            Self::Custom(n) => base + chrono::Duration::days(i64::from(*n)),
        };
        Some(next.to_string())
    }
}

impl fmt::Display for RepeatRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Daily => f.write_str("daily"),
            Self::Weekly => f.write_str("weekly"),
            Self::Monthly => f.write_str("monthly"),
            Self::Yearly => f.write_str("yearly"),
            Self::Weekdays => f.write_str("weekdays"),
            Self::Custom(n) => write!(f, "custom:{n}"),
        }
    }
}

impl FromStr for RepeatRule {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        if s == "daily" {
            Ok(Self::Daily)
        } else if s == "weekly" {
            Ok(Self::Weekly)
        } else if s == "monthly" {
            Ok(Self::Monthly)
        } else if s == "yearly" {
            Ok(Self::Yearly)
        } else if s == "weekdays" {
            Ok(Self::Weekdays)
        } else if let Some(n) = s.strip_prefix("custom:") {
            n.parse::<u32>().map(Self::Custom).map_err(|_| ())
        } else if let Some(d) = s.strip_suffix('d') {
            // e.g. 2d = every 2 days
            d.parse::<u32>().map(Self::Custom).map_err(|_| ())
        } else if let Some(w) = s.strip_suffix('w') {
            // e.g. 3w = every 3 weeks = 21 days
            w.parse::<u32>()
                .ok()
                .and_then(|n| n.checked_mul(7))
                .map(Self::Custom)
                .ok_or(())
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RepeatRule;

    #[test]
    fn from_str_2d_and_3w() {
        assert_eq!("2d".parse::<RepeatRule>().unwrap(), RepeatRule::Custom(2));
        assert_eq!("3w".parse::<RepeatRule>().unwrap(), RepeatRule::Custom(21));
    }

    #[test]
    fn from_str_weekdays() {
        assert_eq!(
            "weekdays".parse::<RepeatRule>().unwrap(),
            RepeatRule::Weekdays
        );
    }
}

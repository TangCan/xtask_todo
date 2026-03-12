//! Tests for `Priority` and `RepeatRule`.

use crate::{Priority, RepeatRule};

#[test]
fn priority_from_str_and_display() {
    assert_eq!("low".parse::<Priority>().unwrap(), Priority::Low);
    assert_eq!("medium".parse::<Priority>().unwrap(), Priority::Medium);
    assert_eq!("high".parse::<Priority>().unwrap(), Priority::High);
    assert_eq!("1".parse::<Priority>().unwrap(), Priority::Low);
    assert_eq!("2".parse::<Priority>().unwrap(), Priority::Medium);
    assert_eq!("3".parse::<Priority>().unwrap(), Priority::High);
    assert_eq!(Priority::Low.as_u8(), 1);
    assert_eq!(Priority::Medium.as_u8(), 2);
    assert_eq!(Priority::High.as_u8(), 3);
    assert_eq!(format!("{}", Priority::Low), "low");
    assert_eq!(format!("{}", Priority::Medium), "medium");
    assert_eq!(format!("{}", Priority::High), "high");
    assert!("invalid".parse::<Priority>().is_err());
}

#[test]
fn repeat_rule_next_due_and_from_str_display() {
    assert_eq!(
        RepeatRule::Daily.next_due_date("2025-01-15"),
        Some("2025-01-16".to_string())
    );
    assert_eq!(
        RepeatRule::Weekly.next_due_date("2025-01-15"),
        Some("2025-01-22".to_string())
    );
    assert_eq!(
        RepeatRule::Monthly.next_due_date("2025-01-15"),
        Some("2025-02-15".to_string())
    );
    assert_eq!(
        RepeatRule::Yearly.next_due_date("2025-01-15"),
        Some("2026-01-15".to_string())
    );
    assert_eq!(
        RepeatRule::Custom(3).next_due_date("2025-01-15"),
        Some("2025-01-18".to_string())
    );
    assert_eq!("daily".parse::<RepeatRule>().unwrap(), RepeatRule::Daily);
    assert_eq!(
        "custom:5".parse::<RepeatRule>().unwrap(),
        RepeatRule::Custom(5)
    );
    assert_eq!(format!("{}", RepeatRule::Daily), "daily");
    assert_eq!(format!("{}", RepeatRule::Weekly), "weekly");
    assert_eq!(format!("{}", RepeatRule::Monthly), "monthly");
    assert_eq!(format!("{}", RepeatRule::Yearly), "yearly");
    assert_eq!(format!("{}", RepeatRule::Weekdays), "weekdays");
    assert_eq!(format!("{}", RepeatRule::Custom(10)), "custom:10");
    assert_eq!("weekly".parse::<RepeatRule>().unwrap(), RepeatRule::Weekly);
    assert_eq!(
        "monthly".parse::<RepeatRule>().unwrap(),
        RepeatRule::Monthly
    );
    assert_eq!("yearly".parse::<RepeatRule>().unwrap(), RepeatRule::Yearly);
    assert!("custom:bad".parse::<RepeatRule>().is_err());
    assert!(RepeatRule::next_due_date(&RepeatRule::Daily, "invalid").is_none());
}

#[test]
fn repeat_rule_weekdays_next_due() {
    // 2025-01-18 is Saturday; next weekday is Monday 2025-01-20
    let next = RepeatRule::Weekdays.next_due_date("2025-01-18");
    assert_eq!(next.as_deref(), Some("2025-01-20"));
}

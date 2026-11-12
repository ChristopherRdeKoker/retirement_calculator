const NO_BREAK_SPACE: char = '\u{a0}';

pub fn group_thousands(digits: &str, separator: char) -> String {
    let mut grouped = String::new();
    for (position, digit) in digits.chars().enumerate() {
        if position > 0 && (digits.len() - position).is_multiple_of(3) {
            grouped.push(separator);
        }
        grouped.push(digit);
    }
    grouped
}

pub fn format_rand(value: f64) -> String {
    if !value.is_finite() {
        return "R —".to_string();
    }
    let rounded = value.abs().round();
    let grouped = group_thousands(&format!("{rounded:.0}"), NO_BREAK_SPACE);
    let sign = if value < 0.0 && rounded > 0.0 {
        "−"
    } else {
        ""
    };
    format!("{sign}R{NO_BREAK_SPACE}{grouped}")
}

pub fn format_compact(value: f64) -> String {
    let size = value.abs();
    let (scaled, suffix) = if size >= 1e12 {
        (value / 1e12, "tn")
    } else if size >= 1e9 {
        (value / 1e9, "bn")
    } else if size >= 1e6 {
        (value / 1e6, "m")
    } else if size >= 1e3 {
        (value / 1e3, "k")
    } else {
        return format!("R{value:.0}");
    };
    let text = if scaled.abs() >= 100.0 || (scaled - scaled.round()).abs() < 0.05 {
        format!("{scaled:.0}")
    } else {
        format!("{scaled:.1}")
    };
    format!("R{text}{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::MAX_MONEY;

    #[test]
    fn group_thousands_inserts_the_separator_every_three_digits() {
        assert_eq!(group_thousands("1234567", ' '), "1 234 567");
        assert_eq!(group_thousands("123456", '.'), "123.456");
        assert_eq!(group_thousands("999", ' '), "999");
        assert_eq!(group_thousands("", ' '), "");
    }

    #[test]
    fn rand_formatting() {
        assert_eq!(format_rand(1_234_567.0), "R\u{a0}1\u{a0}234\u{a0}567");
        assert_eq!(format_rand(999.4), "R\u{a0}999");
        assert_eq!(format_rand(0.0), "R\u{a0}0");
        assert_eq!(format_rand(-1_500.0), "−R\u{a0}1\u{a0}500");
        assert_eq!(format_rand(f64::NAN), "R —");
        assert_eq!(format_compact(2_500_000.0), "R2.5m");
        assert_eq!(format_compact(12_000.0), "R12k");
        assert_eq!(format_compact(0.0), "R0");
    }

    #[test]
    fn rand_rounds_half_away_from_zero_and_hides_negative_zero() {
        assert_eq!(format_rand(0.5), "R\u{a0}1");
        assert_eq!(format_rand(1_499.5), "R\u{a0}1\u{a0}500");
        assert_eq!(format_rand(-0.4), "R\u{a0}0");
        assert_eq!(format_rand(-0.5), "−R\u{a0}1");
        assert_eq!(format_rand(-1_234_567.8), "−R\u{a0}1\u{a0}234\u{a0}568");
    }

    #[test]
    fn rand_groups_large_numbers_with_non_breaking_spaces() {
        assert_eq!(
            format_rand(MAX_MONEY),
            "R\u{a0}1\u{a0}000\u{a0}000\u{a0}000\u{a0}000"
        );
        assert_eq!(format_rand(100_000.0), "R\u{a0}100\u{a0}000");
        assert_eq!(format_rand(1_000.0), "R\u{a0}1\u{a0}000");
        assert_eq!(format_rand(f64::INFINITY), "R —");
        assert!(!format_rand(1e25).contains(' '));
    }

    #[test]
    fn compact_rand_picks_a_suffix_and_one_decimal() {
        assert_eq!(format_compact(999.0), "R999");
        assert_eq!(format_compact(1_000.0), "R1k");
        assert_eq!(format_compact(1_500.0), "R1.5k");
        assert_eq!(format_compact(123_456_000.0), "R123m");
        assert_eq!(format_compact(-2_500_000.0), "R-2.5m");
        assert_eq!(format_compact(3_960_000_000.0), "R4bn");
        assert_eq!(format_compact(MAX_MONEY), "R1tn");
    }
}

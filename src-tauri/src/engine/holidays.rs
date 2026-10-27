use crate::models::HolidayInfo;
use chrono::{Datelike, Duration, NaiveDate};

/// Computes Easter Sunday for any Gregorian year using the Meeus/Jones/Butcher algorithm.
pub fn calculate_easter(year: i32) -> Option<NaiveDate> {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = ((h + l - 7 * m + 114) / 31) as u32;
    let day = (((h + l - 7 * m + 114) % 31) + 1) as u32;

    NaiveDate::from_ymd_opt(year, month, day)
}

/// Returns the Italian National Holiday name if `date` is an official holiday.
pub fn get_italian_holiday_name(date: NaiveDate) -> Option<&'static str> {
    match (date.month(), date.day()) {
        (1, 1) => return Some("Capodanno"),
        (1, 6) => return Some("Epifania"),
        (4, 25) => return Some("Festa della Liberazione"),
        (5, 1) => return Some("Festa del Lavoro"),
        (6, 2) => return Some("Festa della Repubblica"),
        (8, 15) => return Some("Ferragosto"),
        (11, 1) => return Some("Tutti i Santi"),
        (12, 8) => return Some("Immacolata Concezione"),
        (12, 25) => return Some("Natale"),
        (12, 26) => return Some("Santo Stefano"),
        _ => {}
    }

    if let Some(easter) = calculate_easter(date.year()) {
        if date == easter {
            return Some("Pasqua");
        }
        if let Some(easter_monday) = easter.checked_add_signed(Duration::days(1)) {
            if date == easter_monday {
                return Some("Pasquetta");
            }
        }
    }

    None
}

/// Lists all Italian national holidays for a given range of years `[start_year, end_year]`.
pub fn list_holidays_for_years(start_year: i32, end_year: i32) -> Vec<HolidayInfo> {
    let mut out = Vec::new();
    let fixed: &[(u32, u32, &str)] = &[
        (1, 1, "Capodanno"),
        (1, 6, "Epifania"),
        (4, 25, "Festa della Liberazione"),
        (5, 1, "Festa del Lavoro"),
        (6, 2, "Festa della Repubblica"),
        (8, 15, "Ferragosto"),
        (11, 1, "Tutti i Santi"),
        (12, 8, "Immacolata Concezione"),
        (12, 25, "Natale"),
        (12, 26, "Santo Stefano"),
    ];

    for year in start_year..=end_year {
        for &(m, d, name) in fixed {
            if let Some(dt) = NaiveDate::from_ymd_opt(year, m, d) {
                out.push(HolidayInfo {
                    date: dt.format("%Y-%m-%d").to_string(),
                    name: name.to_string(),
                    is_national: true,
                });
            }
        }

        if let Some(easter) = calculate_easter(year) {
            out.push(HolidayInfo {
                date: easter.format("%Y-%m-%d").to_string(),
                name: "Pasqua".to_string(),
                is_national: true,
            });
            if let Some(pasquetta) = easter.checked_add_signed(Duration::days(1)) {
                out.push(HolidayInfo {
                    date: pasquetta.format("%Y-%m-%d").to_string(),
                    name: "Pasquetta".to_string(),
                    is_national: true,
                });
            }
        }
    }

    out.sort_by(|a, b| a.date.cmp(&b.date));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easter_calculation_known_years() {
        assert_eq!(
            calculate_easter(2026),
            NaiveDate::from_ymd_opt(2026, 4, 5)
        );
        assert_eq!(
            calculate_easter(2027),
            NaiveDate::from_ymd_opt(2027, 3, 28)
        );
        assert_eq!(
            calculate_easter(2028),
            NaiveDate::from_ymd_opt(2028, 4, 16)
        );
    }

    #[test]
    fn test_italian_holidays_detection() {
        let epifania = NaiveDate::from_ymd_opt(2027, 1, 6).unwrap();
        assert_eq!(get_italian_holiday_name(epifania), Some("Epifania"));

        let pasquetta_2027 = NaiveDate::from_ymd_opt(2027, 3, 29).unwrap();
        assert_eq!(get_italian_holiday_name(pasquetta_2027), Some("Pasquetta"));

        let ordinary = NaiveDate::from_ymd_opt(2027, 1, 8).unwrap();
        assert_eq!(get_italian_holiday_name(ordinary), None);
    }
}

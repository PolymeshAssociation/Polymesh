use chrono::{Datelike, NaiveDate, NaiveDateTime};
use codec::{Decode, Encode};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::convert::TryFrom;

/// Calendar units for timing recurring operations.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum CalendarUnit {
    /// A unit of one second.
    Second,
    /// A unit of one minute.
    Minute,
    /// A unit of one hour.
    Hour,
    /// A unit of one day.
    Day,
    /// A unit of one week.
    Week,
    /// A unit of one month.
    Month,
    /// A unit of one year.
    Year,
}

impl Default for CalendarUnit {
    fn default() -> Self {
        Self::Second
    }
}

/// A simple period which is a multiple of a `CalendarUnit`.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct CalendarPeriod {
    /// The base calendar unit.
    pub unit: CalendarUnit,
    /// The number of base units in the period.
    pub multiplier: u64,
}

impl CalendarPeriod {
    /// For fixed length calendar periods (in UTC, without leap seconds), computes the number of
    /// seconds they contain. For variable length periods, returns `None`.
    pub fn as_secs(&self) -> Option<u64> {
        match self.unit {
            CalendarUnit::Second => Some(self.multiplier),
            CalendarUnit::Minute => Some(self.multiplier * 60),
            CalendarUnit::Hour => Some(self.multiplier * 60 * 60),
            CalendarUnit::Day => Some(self.multiplier * 60 * 60 * 24),
            CalendarUnit::Week => Some(self.multiplier * 60 * 60 * 24 * 7),
            _ => None,
        }
    }
}

/// The schedule of an asset checkpoint containing the start time `start` and the optional period
/// `period` in case the checkpoint is to recur after `start` at regular intervals.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct CheckpointSchedule {
    /// Unix time in seconds (UTC).
    pub start: u64,
    /// The period at which the checkpoint is set to recur after `start`.
    pub period: CalendarPeriod,
}

impl CheckpointSchedule {
    /// Computes the next checkpoint for the schedule given the current timestamp in seconds UTC.
    pub fn next_checkpoint(&self, now_as_secs_utc: u64) -> Option<u64> {
        if self.start > now_as_secs_utc {
            // The start time is in the future.
            return Some(self.start);
        }
        // The start time is in the past.
        let multiplier = self.period.multiplier;
        if multiplier == 0 {
            // The period is empty while the start time has already passed.
            return None;
        }
        if let Some(period_as_secs) = self.period.as_secs() {
            // The period is of fixed length in seconds UTC.
            let secs_since_start = now_as_secs_utc - self.start;
            let elapsed_periods: u64 = secs_since_start / period_as_secs;
            return Some(secs_since_start + period_as_secs * (elapsed_periods + 1));
        }
        // The period is of variable length.
        let date_time_start = NaiveDateTime::from_timestamp(i64::try_from(self.start).ok()?, 0);
        let date_start = date_time_start.date();
        let year_start = date_start.year();
        let month_start = date_start.month();
        let day_start = date_start.day();
        let date_time_now = NaiveDateTime::from_timestamp(i64::try_from(now_as_secs_utc).ok()?, 0);
        let date_now = date_time_now.date();
        match self.period.unit {
            CalendarUnit::Month => {
                // Convert the multiplier to match the type of month.
                let multiplier = u32::try_from(multiplier).ok()?;
                let year_diff = u32::try_from(date_now.year() - year_start).ok()?;
                // Convert months to base 12.
                let month_start_base12 = month_start - 1;
                let month_now_base12 = date_now.month() - 1;
                let elapsed_months = year_diff * 12 + month_now_base12 - month_start_base12;
                let elapsed_periods = elapsed_months / multiplier;
                let next_period_months = multiplier * (elapsed_periods + 1);
                // The month of the next period counting from the beginning of `year_start`.
                let denormalized_next_period_month = month_start_base12 + next_period_months;
                let next_period_year =
                    year_start + i32::try_from(denormalized_next_period_month / 12).ok()?;
                // Month in base 12.
                let next_period_month = denormalized_next_period_month % 12;
                let next_period_day = if matches!(next_period_month, 3 | 5 | 8 | 10) {
                    // Handle 30-day months.
                    u32::min(day_start, 30)
                } else if next_period_month == 1 {
                    // Handle February
                    if is_leap_year(next_period_year) {
                        u32::min(day_start, 29)
                    } else {
                        u32::min(day_start, 28)
                    }
                } else {
                    day_start
                };
                let date_next = NaiveDate::from_ymd(
                    i32::try_from(next_period_year).ok()?,
                    // Convert months back to calendar numbering.
                    next_period_month + 1,
                    next_period_day,
                );
                Some(u64::try_from(date_next.and_time(date_time_start.time()).timestamp()).ok()?)
            }
            CalendarUnit::Year => {
                // Convert the multiplier to match the type of year.
                let multiplier = i32::try_from(multiplier).ok()?;
                let elapsed_periods: i32 = (date_now.year() - year_start) / multiplier;
                let next_period_year = year_start + multiplier * (elapsed_periods + 1);
                let next_period_day = if month_start == 2 && !is_leap_year(next_period_year) {
                    // Handle February in non-leap years.
                    u32::min(day_start, 28)
                } else {
                    day_start
                };
                let date_next = NaiveDate::from_ymd(
                    next_period_year,
                    // Convert months back to calendar numbering.
                    month_start,
                    next_period_day,
                );
                Some(u64::try_from(date_next.and_time(date_time_start.time()).timestamp()).ok()?)
            }
            _ => unreachable!(),
        }
    }
}

/// Copy of a function from the `time` crate.
///
/// Returns if the provided year is a leap year in the proleptic Gregorian calendar. Uses
/// [astronomical year numbering](https://en.wikipedia.org/wiki/Astronomical_year_numbering).
///
/// ```rust
/// use polymesh_primitives::calendar::is_leap_year;
/// assert!(!is_leap_year(1900));
/// assert!(is_leap_year(2000));
/// assert!(is_leap_year(2004));
/// assert!(!is_leap_year(2005));
/// assert!(!is_leap_year(2100));
/// ```
#[inline(always)]
pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0) & ((year % 100 != 0) | (year % 400 == 0))
}

#[cfg(test)]
mod tests {
    use super::{CalendarPeriod, CalendarUnit, CheckpointSchedule};
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

    fn format_date_time(timestamp: i64) {
        format!("{}", NaiveDateTime::from_timestamp(timestamp, 0));
    }

    #[test]
    fn next_checkpoint_seconds_test() {
        let period_day_seconds = CalendarPeriod {
            unit: CalendarUnit::Second,
            multiplier: 60 * 60 * 24,
        };
        let schedule_day_seconds = CheckpointSchedule {
            start: 60 * 60, // 1:00:00
            period: period_day_seconds,
        };
        let checkpoint = schedule_day_seconds.next_checkpoint(
            NaiveDate::from_ymd(2020, 12, 12)
                .and_time(NaiveTime::from_hms(1, 2, 3))
                .timestamp() as u64,
        );
        assert_eq!(
            format_date_time(checkpoint.unwrap() as i64),
            format_date_time(
                NaiveDate::from_ymd(2020, 12, 13)
                    .and_time(NaiveTime::from_hms(13, 0, 0))
                    .timestamp()
            )
        );
    }

    #[test]
    fn next_checkpoint_months_test() {
        let period_5_months = CalendarPeriod {
            unit: CalendarUnit::Month,
            multiplier: 5,
        };
        let schedule_5_months = CheckpointSchedule {
            start: 0,
            period: period_5_months,
        };
        let checkpoint = schedule_5_months.next_checkpoint(
            NaiveDate::from_ymd(1970, 4, 1)
                .and_time(NaiveTime::from_hms(1, 2, 3))
                .timestamp() as u64,
        );
        assert_eq!(
            format_date_time(checkpoint.unwrap() as i64),
            format_date_time(
                NaiveDate::from_ymd(1970, 5, 1)
                    .and_time(NaiveTime::from_hms(0, 0, 0))
                    .timestamp()
            )
        );
    }

    #[test]
    fn next_checkpoint_end_of_month_test() {
        let period_1_month = CalendarPeriod {
            unit: CalendarUnit::Month,
            multiplier: 1,
        };
        let schedule_end_of_month = CheckpointSchedule {
            start: NaiveDate::from_ymd(2024, 1, 31)
                .and_time(NaiveTime::from_hms(1, 2, 3))
                .timestamp() as u64,
            period: period_1_month,
        };
        let checkpoint_leap_feb = schedule_end_of_month.next_checkpoint(
            NaiveDate::from_ymd(2024, 1, 31)
                .and_time(NaiveTime::from_hms(1, 2, 30))
                .timestamp() as u64,
        );
        assert_eq!(
            format_date_time(checkpoint_leap_feb.unwrap() as i64),
            format_date_time(
                NaiveDate::from_ymd(2024, 2, 29)
                    .and_time(NaiveTime::from_hms(1, 2, 3))
                    .timestamp()
            )
        );
        let checkpoint_nonleap_feb = schedule_end_of_month.next_checkpoint(
            NaiveDate::from_ymd(2025, 1, 31)
                .and_time(NaiveTime::from_hms(1, 2, 30))
                .timestamp() as u64,
        );
        assert_eq!(
            format_date_time(checkpoint_nonleap_feb.unwrap() as i64),
            format_date_time(
                NaiveDate::from_ymd(2024, 2, 28)
                    .and_time(NaiveTime::from_hms(1, 2, 3))
                    .timestamp()
            )
        );
        let checkpoint_apr = schedule_end_of_month.next_checkpoint(
            NaiveDate::from_ymd(2025, 3, 31)
                .and_time(NaiveTime::from_hms(1, 2, 30))
                .timestamp() as u64,
        );
        assert_eq!(
            format_date_time(checkpoint_apr.unwrap() as i64),
            format_date_time(
                NaiveDate::from_ymd(2025, 4, 30)
                    .and_time(NaiveTime::from_hms(1, 2, 3))
                    .timestamp()
            )
        );
    }
}

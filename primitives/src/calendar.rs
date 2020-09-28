use chrono::{Datelike, NaiveDate, NaiveDateTime};
use codec::{Decode, Encode};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::convert::TryFrom;
use time::util::is_leap_year;

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
        let date_time_now = NaiveDateTime::from_timestamp(i64::try_from(now_as_secs_utc).ok()?, 0);
        let date_now = date_time_now.date();
        match self.period.unit {
            CalendarUnit::Month => {
                // Convert the multiplier to match the type of month.
                let multiplier = u32::try_from(multiplier).ok()?;
                let day_start = date_time_start.day();
                let year_start = date_start.year();
                let year_now = date_now.year();
                let year_diff = u32::try_from(year_now - year_start).ok()?;
                // Convert months to base 12.
                let month_start = date_start.month() - 1;
                let month_now = date_now.month() - 1;
                let elapsed_months = year_diff * 12 + month_now - month_start;
                let elapsed_periods = elapsed_months / multiplier;
                let next_period_months = multiplier * (elapsed_periods + 1);
                // The month of the next period counting from the beginning of `year_start`.
                let denormalized_next_period_month = month_start + next_period_months;
                let next_period_year =
                    year_start + i32::try_from(denormalized_next_period_month / 12).ok()?;
                let next_period_month = denormalized_next_period_month % 12;
                let next_period_day =
                    if matches!(next_period_month, 3 | 5 | 8 | 10) && day_start == 31 {
                        // Handle 30-day months.
                        30
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
                NaiveDateTime::new(date_next, date_time_start.time())
                    .map(|dt| dt.timestamp() as u64)
            }
            CalendarUnit::Year => {
                // Convert the multiplier to match the type of year.
                let multiplier = i32::try_from(multiplier).unwrap_or(i32::MAX);
                let elapsed_periods: i32 = (date_now.year() - date_start.year()) / multiplier;
                let next_period_year = date_start.year() + multiplier * (elapsed_periods + 1);
                date_time_start
                    .with_year(next_period_year)
                    .or_else(|| {
                        // Handle the case of 29 February by trying the previous day.
                        if date_time_start.month() == 2 && date_time_start.day() == 29 {
                            date_time_start
                                .with_day(28)
                                .and_then(|dt| dt.with_year(next_period_year))
                        } else {
                            None
                        }
                    })
                    .map(|dt| dt.timestamp() as u64)
            }
            _ => unreachable!(),
        }
    }
}

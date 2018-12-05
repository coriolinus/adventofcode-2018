extern crate chrono;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate itertools;
extern crate text_io;

use chrono::{Duration, NaiveDateTime as DateTime, Timelike};
use itertools::Itertools;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use text_io::try_scan;

pub type Guard = u32;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Event {
    BeginShift(Guard),
    FallsAsleep,
    WakesUp,
}

impl FromStr for Event {
    type Err = text_io::Error;

    fn from_str(s: &str) -> Result<Event, Self::Err> {
        use Event::*;
        match s {
            "falls asleep" => Ok(FallsAsleep),
            "wakes up" => Ok(WakesUp),
            s => {
                let id: u32;
                try_scan!(s.bytes() => "Guard #{} begins shift", id);
                Ok(BeginShift(id))
            }
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use Event::*;
        match self {
            FallsAsleep => write!(f, "falls asleep"),
            WakesUp => write!(f, "wakes up"),
            BeginShift(id) => write!(f, "Guard #{} begins shift", id),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Record {
    pub timestamp: DateTime,
    pub event: Event,
}

impl PartialOrd for Record {
    fn partial_cmp(&self, other: &Record) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Record {
    fn cmp(&self, other: &Record) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

#[derive(Debug, Fail)]
pub enum ParseError {
    #[fail(display = "parsing datetime: {}", _0)]
    ParseDateTime(#[cause] chrono::format::ParseError),
    #[fail(display = "parsing event: {}", _0)]
    ParseEvent(#[cause] text_io::Error),
}

impl From<chrono::format::ParseError> for ParseError {
    fn from(err: chrono::format::ParseError) -> ParseError {
        ParseError::ParseDateTime(err)
    }
}

impl From<text_io::Error> for ParseError {
    fn from(err: text_io::Error) -> ParseError {
        ParseError::ParseEvent(err)
    }
}

impl FromStr for Record {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Record, Self::Err> {
        // [1518-07-31 00:54] wakes up
        let timestamp = DateTime::parse_from_str(&s[1..17], "%Y-%m-%d %H:%M")?;
        let event = Event::from_str(&s[19..])?;
        Ok(Record { timestamp, event })
    }
}

impl Record {
    pub fn sleep_minutes(records: &[Self]) -> Option<Sleepytime> {
        // check to see if self is sorted
        for w in records.windows(2) {
            if w[0] > w[1] {
                return None;
            }
        }

        Some(Sleepytime {
            records,
            ..Sleepytime::default()
        })
    }

    pub fn most_minutes_asleep(records: &[Self]) -> Option<Guard> {
        let mut sleepmap = HashMap::new();
        for (guard, _) in Record::sleep_minutes(records)? {
            *sleepmap.entry(guard).or_default() += 1;
        }

        // find max
        sleepmap
            .iter()
            .map(|(guard, minutes): (&Guard, &Minute)| (minutes, guard))
            .max()
            .map(|(_, &g)| g)
    }

    pub fn most_sleepy_minute(records: &[Self], guard: Guard) -> Option<Minute> {
        let mut sleepmap = HashMap::new();
        for moment in
            Record::sleep_minutes(records)?
                .filter_map(|(g, m)| if g == guard { Some(m) } else { None })
        {
            let minute = minute_of(&moment);
            *sleepmap.entry(minute).or_default() += 1;
        }

        // find max
        sleepmap
            .iter()
            .map(|(minute, count): (&Minute, &usize)| (count, minute))
            .max()
            .map(|(_, &minute)| minute)
    }
}

pub type Minute = u32;

fn minute_of(d: &DateTime) -> Minute {
    d.time().minute() as Minute
}

#[derive(Default)]
pub struct Sleepytime<'a> {
    records: &'a [Record],
    record_idx: usize,
    guard: Option<Guard>,
    moment: Option<DateTime>,
}

impl<'a> Iterator for Sleepytime<'a> {
    type Item = (Guard, DateTime);

    fn next(&mut self) -> Option<(Guard, DateTime)> {
        loop {
            if self.record_idx >= self.records.len() {
                return None;
            }
            match self.records[self.record_idx] {
                Record {
                    event: Event::BeginShift(guard),
                    ..
                } => {
                    if self.guard.is_some() && self.moment.is_some() {
                        panic!("guard {:?} never woke up", self.guard);
                    }
                    self.guard = Some(guard);
                }
                Record {
                    timestamp,
                    event: Event::FallsAsleep,
                } => self.moment = Some(timestamp),
                Record {
                    timestamp,
                    event: Event::WakesUp,
                } => {
                    // this is where actual iteration happens
                    let guard = self
                        .guard
                        .expect("malformed input: no shift change before wakeup");
                    let moment = self
                        .moment
                        .expect("malformed input: didn't sleep before wakeup");

                    if moment < timestamp {
                        let minute = moment.clone();
                        self.moment = Some(moment + Duration::minutes(1));
                        return Some((guard, minute));
                    } else {
                        self.moment = None;
                    }
                }
            }
            self.record_idx += 1;
        }
    }
}

const DBG_HEADER: &str = "\
Date   ID    Minute
             000000000011111111112222222222333333333344444444445555555555
             012345678901234567890123456789012345678901234567890123456789\
";

impl Record {
    pub fn debug<W: std::io::Write>(w: &mut W, records: &[Self]) {
        writeln!(w, "{}", DBG_HEADER).unwrap();
        for ((guard, date), moments) in &Record::sleep_minutes(records)
            .unwrap()
            .group_by(|(g, m)| (g.clone(), m.format("%m-%d").to_string()))
        {
            write!(w, "{:6} #{:4} ", date, guard);

            use itertools::EitherOrBoth::{Both, Left, Right};
            for c in moments
                .merge_join_by(0..60, |i, j| minute_of(&i.1).cmp(j))
                .map(|either| match either {
                    Left(_) => unreachable!(),
                    Right(_) => '.',
                    Both(_, _) => '#',
                }) {
                write!(w, "{}", c);
            }
            writeln!(w, "");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_record() {
        let example = "[1518-07-31 00:54] wakes up";
        let expect = Record {
            timestamp: DateTime::parse_from_str("1518-07-31 00:54", "%Y-%m-%d %H:%M").unwrap(),
            event: Event::WakesUp,
        };
        assert_eq!(
            expect,
            Record::from_str(example).expect("record parse must succeed")
        );
    }
}

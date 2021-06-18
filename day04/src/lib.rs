use aoclib::parse;

use std::{collections::HashMap, path::Path};

type Id = u32;

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    parse_display::FromStr,
    parse_display::Display,
)]
#[display("{year}-{month}-{day} {hour}:{minute}")]
struct Timestamp {
    year: i16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    parse_display::FromStr,
    parse_display::Display,
)]
enum Action {
    #[display("Guard #{0} begins shift")]
    BeginShift(Id),
    #[display("falls asleep")]
    FallAsleep,
    #[display("wakes up")]
    WakeUp,
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    parse_display::FromStr,
    parse_display::Display,
)]
#[display("[{timestamp}] {action}")]
struct LogEntry {
    timestamp: Timestamp,
    action: Action,
}

fn minutes_asleep(logs: &[LogEntry]) -> HashMap<Id, u32> {
    let mut sleep_time = HashMap::new();
    let mut guard = None;
    let mut sleep_start = None;

    for entry in logs {
        match entry.action {
            Action::BeginShift(id) => {
                debug_assert!(sleep_start.is_none());
                guard = Some(id);
            }
            Action::FallAsleep => {
                debug_assert!(sleep_start.is_none());
                sleep_start = entry.timestamp.minute.into();
            }
            Action::WakeUp => {
                let sleep_start = sleep_start.take().expect("can't wake if not asleep");
                let sleep_end = entry.timestamp.minute;
                *sleep_time
                    .entry(guard.expect("can't wake if not a guard"))
                    .or_default() += (sleep_end - sleep_start) as u32;
            }
        }
    }

    sleep_time
}

fn sleepiest_minute(guard: Id, logs: &[LogEntry]) -> u32 {
    let mut minutes_slept = [0; 60];
    let mut on_shift = false;
    let mut sleep_start = None;

    for entry in logs {
        match entry.action {
            Action::BeginShift(id) => on_shift = id == guard,
            Action::FallAsleep if on_shift => {
                debug_assert!(sleep_start.is_none());
                sleep_start = entry.timestamp.minute.into();
            }
            Action::WakeUp if on_shift => {
                let sleep_start = sleep_start.take().expect("can't wake if not asleep");
                let sleep_end = entry.timestamp.minute;
                for minute in sleep_start..sleep_end {
                    minutes_slept[minute as usize] += 1;
                }
            }
            _ => {}
        }
    }

    std::array::IntoIter::new(minutes_slept)
        .enumerate()
        .map(|(idx, time_slept)| (time_slept, idx))
        .max()
        .map(|(_, idx)| idx as u32)
        .unwrap()
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let mut logs: Vec<LogEntry> = parse(input)?.collect();
    logs.sort_unstable();

    let sleep_times = minutes_asleep(&logs);
    let sleepiest_guard = sleep_times
        .iter()
        .map(|(&id, &slept)| (slept, id))
        .max()
        .map(|(_, id)| id)
        .ok_or(Error::NoSolution)?;
    let sleepiest_minute = sleepiest_minute(sleepiest_guard, &logs);
    dbg!(sleepiest_guard, sleepiest_minute, sleepiest_guard * sleepiest_minute);
    Ok(())
}

pub fn part2(_input: &Path) -> Result<(), Error> {
    unimplemented!()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("No solution found")]
    NoSolution,
}

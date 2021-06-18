use aoclib::parse;
use std::{collections::HashMap, convert::TryInto, path::Path};

type Id = u32;
type Minute = u32;

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

type AsleepByMinute = [HashMap<Id, Minute>; 60];

/// Produce a data structure recording for each minute, how many times each guard was asleep that minute.
fn asleep_by_minute(logs: &[LogEntry]) -> AsleepByMinute {
    let asleep = vec![HashMap::default(); 60];
    let mut asleep: [HashMap<_, _>; 60] =
        asleep.try_into().expect("just initialized; never changed");
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
                let guard = guard.expect("can't wake if not a guard");
                for minute in sleep_start..sleep_end {
                    *asleep[minute as usize].entry(guard).or_default() += 1;
                }
            }
        }
    }

    asleep
}

/// count total minutes asleep per guard
fn total_minutes_by_guard(by_minute: &AsleepByMinute) -> HashMap<Id, Minute> {
    by_minute.iter().fold(HashMap::new(), |mut acc, elem| {
        for (guard, times_slept) in elem {
            *acc.entry(*guard).or_default() += times_slept;
        }
        acc
    })
}

/// get guard with most minutes asleep
fn sleepiest_guard(by_guard: &HashMap<Id, Minute>) -> Option<Id> {
    by_guard
        .iter()
        .map(|(&id, &slept)| (slept, id))
        .max()
        .map(|(_, id)| id)
}

/// get minute with most instances asleep for a guard
fn sleepiest_minute(guard: Id, by_minute: &AsleepByMinute) -> Minute {
    by_minute
        .iter()
        .enumerate()
        .map(|(minute, map)| (map.get(&guard).copied().unwrap_or_default(), minute))
        .max()
        .map(|(_, minute)| minute as Minute)
        .expect("60 elements; never empty")
}

/// get guard most frequently asleep on the same minute
fn most_freq_asleep_per_minute(by_minute: &AsleepByMinute) -> Option<(Id, Minute)> {
    by_minute
        .iter()
        .enumerate()
        .flat_map(|(minute, by_guard)| {
            by_guard
                .iter()
                .map(move |(&guard, &minutes_asleep)| (minutes_asleep, guard, minute as Minute))
        })
        .max()
        .map(|(_, guard, minute)| (guard, minute))
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let mut logs: Vec<LogEntry> = parse(input)?.collect();
    logs.sort_unstable();

    let by_minute = asleep_by_minute(&logs);
    let sleep_times = total_minutes_by_guard(&by_minute);
    let sleepiest_guard = sleepiest_guard(&sleep_times).ok_or(Error::NoSolution)?;
    let sleepiest_minute = sleepiest_minute(sleepiest_guard, &by_minute);
    println!("part 1");
    dbg!(
        sleepiest_guard,
        sleepiest_minute,
        sleepiest_guard * sleepiest_minute
    );
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let mut logs: Vec<LogEntry> = parse(input)?.collect();
    logs.sort_unstable();

    let by_minute = asleep_by_minute(&logs);
    let (guard, minute) = most_freq_asleep_per_minute(&by_minute).ok_or(Error::NoSolution)?;
    println!("part 2");
    dbg!(guard, minute, guard * minute);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("No solution found")]
    NoSolution,
}

use aoclib::parse;
use std::{
    cmp::Reverse,
    collections::{BTreeSet, BinaryHeap, HashMap},
    path::Path,
    str::FromStr,
};
use text_io::try_scan;

pub type Step = char;
pub type Seconds = u32;

pub const N_WORKERS: usize = 5;
pub const TASK_BASE_DURATION: Seconds = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge {
    prereq: Step,
    blocked: Step,
}

impl FromStr for Edge {
    type Err = text_io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (prereq, blocked): (Step, Step);
        try_scan!(s.bytes() => "Step {} must be finished before step {} can begin.", prereq, blocked);
        Ok(Edge { prereq, blocked })
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Node {
    prereq: BTreeSet<Step>,
    blocked: BTreeSet<Step>,
}

/// a graph lists, for each step, all of its prerequisites in sorted order
pub type Graph = HashMap<Step, Node>;

fn make_graph(edges: &[Edge]) -> Graph {
    let mut graph = Graph::new();

    for edge in edges {
        graph
            .entry(edge.blocked)
            .or_default()
            .prereq
            .insert(edge.prereq);
        graph
            .entry(edge.prereq)
            .or_default()
            .blocked
            .insert(edge.blocked);
    }

    graph
}

fn no_prerequisites(graph: &Graph) -> impl '_ + Iterator<Item = Step> {
    graph
        .iter()
        .filter(|(_step, node)| node.prereq.is_empty())
        .map(|(&step, _node)| step)
}

fn topo_sort(mut graph: Graph) -> Vec<Step> {
    let mut out = Vec::with_capacity(graph.len());

    let mut ready: BinaryHeap<_> = no_prerequisites(&graph).map(|step| Reverse(step)).collect();

    while let Some(Reverse(step)) = ready.pop() {
        out.push(step);
        if let Some(node) = graph.remove(&step) {
            for was_blocked in node.blocked {
                if let Some(wb_node) = graph.get_mut(&was_blocked) {
                    wb_node.prereq.remove(&step);
                    if wb_node.prereq.is_empty() {
                        ready.push(Reverse(was_blocked));
                    }
                }
            }
        }
    }

    // double-check ourselves
    debug_assert!(graph.is_empty());

    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Event {
    CompleteTask(Seconds),    // unblocks a worker
    Unblocked(Seconds, Step), // when a task becomes available
}

impl Event {
    fn time(self) -> Seconds {
        match self {
            Event::CompleteTask(t) => t,
            Event::Unblocked(t, _) => t,
        }
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Event) -> std::cmp::Ordering {
        use crate::Event::*;
        use std::cmp::Ordering::*;
        match (self, other) {
            (CompleteTask(s), CompleteTask(o)) => s.cmp(o),
            (Unblocked(st, ss), Unblocked(ot, os)) => (st, ss).cmp(&(ot, os)),
            // workers are less than tasks, all else being equal, meaning
            // that they unblock before new tasks become available
            (CompleteTask(s), Unblocked(o, _)) => s.cmp(o).then(Less),
            (Unblocked(s, _), CompleteTask(o)) => s.cmp(o).then(Greater),
        }
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Event) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn make_duration_of(duration_base: Seconds) -> impl Fn(Step) -> Seconds {
    move |step| duration_base + 1 + (step as u8 - 'A' as u8) as Seconds
}

fn assembly_time(graph: Graph) -> Seconds {
    let duration_of = make_duration_of(TASK_BASE_DURATION);
    assembly_time_with(graph, N_WORKERS, duration_of)
}

fn assembly_time_with(
    mut graph: Graph,
    workers: usize,
    duration_of: impl Fn(Step) -> Seconds,
) -> Seconds {
    let mut time = 0;
    let mut workers_working = 0;

    // ready: Heap<Event>
    let mut ready: BinaryHeap<_> = no_prerequisites(&graph)
        .map(|step| Reverse(Event::Unblocked(0, step)))
        .collect();

    while let Some(Reverse(event)) = ready.pop() {
        match event {
            Event::CompleteTask(t) => {
                time = t;
                workers_working -= 1;
            }
            Event::Unblocked(t, step) => {
                time = t;

                debug_assert!(
                    workers_working <= workers,
                    "can't have imaginary workers working"
                );
                if workers_working == workers {
                    // no workers available
                    // reset and try again after the next event
                    let Reverse(next_event) = ready
                        .peek()
                        .expect("if all workers are occupied, there must be more events");
                    let next_time = next_event.time();
                    ready.push(Reverse(Event::Unblocked(next_time, step)));
                    continue;
                }

                if let Some(node) = graph.remove(&step) {
                    let finish = time + duration_of(step);

                    workers_working += 1;
                    ready.push(Reverse(Event::CompleteTask(finish)));

                    for was_blocked in node.blocked {
                        if let Some(wb_node) = graph.get_mut(&was_blocked) {
                            wb_node.prereq.remove(&step);
                            if wb_node.prereq.is_empty() {
                                ready.push(Reverse(Event::Unblocked(finish, was_blocked)));
                            }
                        }
                    }
                }
            }
        }
    }

    time
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let edges: Vec<Edge> = parse(input)?.collect();
    let graph = make_graph(&edges);
    let sorted_steps: String = topo_sort(graph).into_iter().collect();

    println!("instruction order: {}", sorted_steps);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let edges: Vec<Edge> = parse(input)?.collect();
    let graph = make_graph(&edges);
    let assembly_time = assembly_time(graph);

    println!("assembly time: {}", assembly_time);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

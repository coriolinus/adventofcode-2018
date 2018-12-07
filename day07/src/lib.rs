use std::cmp::Reverse;
use std::collections::{BTreeSet, BinaryHeap, HashMap};
use std::str::FromStr;
use text_io::try_scan;

pub type Step = char;

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

pub fn as_graph(edges: &[Edge]) -> Graph {
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

pub fn topo_sort(graph: &Graph) -> Vec<Step> {
    // #[cfg(feature = "debug_out")]
    // println!("graph: {:#?}", graph);

    let mut graph = graph.clone();
    let mut out = Vec::with_capacity(graph.len());

    let mut ready: BinaryHeap<_> = graph
        .iter()
        .filter_map(|(step, node)| {
            if node.prereq.is_empty() {
                Some(Reverse(*step))
            } else {
                None
            }
        })
        .collect();

    #[cfg(feature = "debug_out")]
    println!("ready: {:?}", ready);

    while let Some(Reverse(step)) = ready.pop() {
        #[cfg(feature = "debug_out")]
        print!("pop {}", step);

        out.push(step);
        if let Some(node) = graph.remove(&step) {
            #[cfg(feature = "debug_out")]
            print!(" blocking {:?}", node.blocked);

            for was_blocked in node.blocked {
                if let Some(wb_node) = graph.get_mut(&was_blocked) {
                    wb_node.prereq.remove(&step);
                    if wb_node.prereq.is_empty() {
                        ready.push(Reverse(was_blocked));
                    }
                }
            }
        }

        #[cfg(feature = "debug_out")]
        println!("");
    }

    // double-check ourselves
    #[cfg(feature = "debug_out")]
    {
        println!("out:   {:?}", out);
        println!("graph: {:#?}", graph);
    }
    debug_assert!(graph.is_empty());

    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Event {
    CompleteTask(u32),    // unblocks a worker
    Unblocked(u32, Step), // when a task becomes available
}

impl Event {
    fn time(&self) -> u32 {
        use crate::Event::*;
        match self {
            CompleteTask(t) => *t,
            Unblocked(t, _) => *t,
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

fn seconds(step: Step) -> u32 {
    61 + (step as u8 - 'A' as u8) as u32
}

pub fn assembly_time(graph: &Graph) -> u32 {
    assembly_time_with(graph, 5, seconds)
}

pub fn assembly_time_with<D>(graph: &Graph, workers: usize, duration: D) -> u32
where
    D: Fn(Step) -> u32,
{
    let mut graph = graph.clone();
    let mut time = 0;
    let mut workers_working = 0;

    use crate::Event::*;

    // ready: Heap<Event>
    let mut ready: BinaryHeap<Reverse<Event>> = graph
        .iter()
        .filter_map(|(step, node)| {
            if node.prereq.is_empty() {
                Some(Reverse(Unblocked(0, *step)))
            } else {
                None
            }
        })
        .collect();

    while let Some(Reverse(event)) = ready.pop() {
        match event {
            CompleteTask(t) => {
                time = t;
                workers_working -= 1;
            }
            Unblocked(t, step) => {
                time = t;

                if workers_working >= workers {
                    // no workers available
                    // reset and try again after the next event
                    let next_time = if let Some(Reverse(event)) = ready.peek() {
                        event.time()
                    } else {
                        // we shouldn't ever hit this branch, but why not, right?
                        time + 1
                    };
                    ready.push(Reverse(Unblocked(next_time, step)));
                    continue;
                }

                if let Some(node) = graph.remove(&step) {
                    let finish = time + duration(step);

                    workers_working += 1;
                    ready.push(Reverse(CompleteTask(finish)));

                    for was_blocked in node.blocked {
                        if let Some(wb_node) = graph.get_mut(&was_blocked) {
                            wb_node.prereq.remove(&step);
                            if wb_node.prereq.is_empty() {
                                ready.push(Reverse(Unblocked(finish, was_blocked)));
                            }
                        }
                    }
                }
            }
        }
    }

    time
}

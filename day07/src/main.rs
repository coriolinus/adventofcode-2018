use day07::{as_graph, assembly_time, topo_sort, Edge};
use util::get_input_lines_as;

fn main() -> Result<(), failure::Error> {
    let edges = get_input_lines_as::<Edge>()?;
    let graph = as_graph(&edges);
    let topo = topo_sort(&graph);
    let topo_s: String = topo.iter().collect();
    println!("sorted deps:   {}", topo_s);
    println!("assembly time: {}", assembly_time(&graph));

    Ok(())
}

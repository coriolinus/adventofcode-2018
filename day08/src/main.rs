use day08::Node;

fn main() -> Result<(), failure::Error> {
    let input = day08::parse_input(&util::get_input_string()?)?;
    let (node, leftovers) = Node::try_parse(&input)
        .ok_or_else(|| failure::format_err!("failed to parse input as nodes"))?;
    debug_assert!(leftovers.is_empty());
    println!("sum of metadata: {}", node.sum_metadata());
    println!("root value:      {}", node.value());
    Ok(())
}

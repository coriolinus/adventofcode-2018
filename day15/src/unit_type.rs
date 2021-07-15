#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    parse_display::FromStr,
    parse_display::Display,
)]
pub(crate) enum UnitType {
    #[display("G")]
    Goblin,
    #[display("E")]
    Elf,
}

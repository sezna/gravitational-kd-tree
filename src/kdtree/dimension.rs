
#[derive(Clone, PartialEq)]
/// Used to represent which dimension the KDTree node has split on.
pub enum Dimension {
    X,
    Y,
    Z,
    None,
}

/// Convenience function that returns the Dimension as a &str.
impl Dimension {
    pub fn as_string(&self) -> &str {
        match *self {
            Dimension::X => return "X",
            Dimension::Y => return "Y",
            Dimension::Z => return "Z",
            _ => return "None",
        }
    }
}

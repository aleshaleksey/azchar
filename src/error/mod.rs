// This module exists to very simply convert errors.
pub fn ma<D: std::fmt::Debug>(input: D) -> String {
    format!("{:?}", input)
}

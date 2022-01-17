#![allow(clippy::module_inception)]
pub mod attribute;
pub mod character;
#[cfg(test)]
mod tests;

pub use attribute::Attribute;
pub use character::Character;

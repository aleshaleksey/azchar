#![allow(clippy::module_inception)]
pub(crate) mod attribute;
pub(crate) mod character;
#[cfg(test)]
mod tests;

pub use attribute::{Attribute, NewAttribute};
pub use character::{Character, NewCharacter};

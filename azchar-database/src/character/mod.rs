#![allow(clippy::module_inception)]
pub mod attribute;
pub mod character;
pub mod image;
pub mod note;
#[cfg(test)]
pub(crate) mod tests;

pub use attribute::{Attribute, NewAttribute};
pub use character::{Character, NewCharacter};
// pub use note::{Image, NewImage};
pub use note::{InputNote, Note};

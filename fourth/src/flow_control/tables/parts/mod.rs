pub(crate) mod attr_option;
pub(crate) mod part_option;
pub(crate) mod parts;

pub(crate) use attr_option::AttrOption;
pub(crate) use part_option::PartOption;

pub(self) const LABEL_SIZE: [f32; 2] = [200., 21.];
pub(self) const SMALL_LABEL_SIZE: [f32; 2] = [60., 21.];

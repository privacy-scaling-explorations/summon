mod extract_constants;
pub mod kal;
mod optimize;
mod reduce_instructions;
mod remove_meta_lines;
mod remove_unused_labels;
mod remove_unused_registers;
mod shake_tree;
mod simplify;
mod simplify_jumps;
pub mod try_to_kal;

pub use optimize::optimize;

pub mod datatypes;
mod internal_datatypes;
mod buffer_writing;
mod viewport_process;
mod discretization;

pub use discretization::discretize;
pub use datatypes::*;
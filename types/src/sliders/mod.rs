mod common;

#[cfg(not(feature = "bmi2"))]
mod magic;
#[cfg(feature = "bmi2")]
mod pext;

pub use common::*;

#[cfg(not(feature = "bmi2"))]
pub use magic::*;
#[cfg(feature = "bmi2")]
pub use pext::*;

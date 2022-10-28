//! A future-based worker that for each input, one output is produced.

mod registrar;
mod traits;
mod worker;

pub use registrar::OneshotRegistrar;
pub use traits::Oneshot;

use std::future::Future;

use super::ReactorScoped;

/// A reactor worker.
pub trait Reactor: Future<Output = ()> {
    /// The Reactor Scope
    type Scope: ReactorScoped;

    /// Creates a reactor worker.
    fn create(scope: Self::Scope) -> Self;
}

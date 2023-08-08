use std::borrow::Cow;

use crate::listener::HistoryListener;
use crate::location::Location;
#[cfg(feature = "query")]
use crate::{error::HistoryResult, query::ToQuery};

/// A trait to provide [`History`] access.
///
/// # Warning
///
/// The behaviour of this trait is not well-defined when you mix multiple history kinds in the same application
/// or use `window().history()` to update session history.
pub trait History: Clone + PartialEq {
    /// Returns the number of elements in [`History`].
    fn len(&self) -> usize;

    /// Returns true if the current [`History`] is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Moves back 1 page in [`History`].
    fn back(&self) {
        self.go(-1);
    }

    /// Moves forward 1 page in [`History`].
    fn forward(&self) {
        self.go(1);
    }

    /// Loads a specific page in [`History`] with a `delta` relative to current page.
    ///
    /// See: <https://developer.mozilla.org/en-US/docs/Web/API/History/go>
    fn go(&self, delta: isize);

    /// Pushes a route entry with [`None`] being the state.
    fn push<'a>(&self, route: impl Into<Cow<'a, str>>);

    /// Replaces the current history entry with provided route and [`None`] state.
    fn replace<'a>(&self, route: impl Into<Cow<'a, str>>);

    /// Pushes a route entry with state.
    fn push_with_state<'a, T>(&self, route: impl Into<Cow<'a, str>>, state: T)
    where
        T: 'static;

    /// Replaces the current history entry with provided route and state.
    fn replace_with_state<'a, T>(&self, route: impl Into<Cow<'a, str>>, state: T)
    where
        T: 'static;

    /// Same as `.push()` but affix the queries to the end of the route.
    #[cfg(feature = "query")]
    fn push_with_query<'a, Q>(
        &self,
        route: impl Into<Cow<'a, str>>,
        query: Q,
    ) -> HistoryResult<(), Q::Error>
    where
        Q: ToQuery;

    /// Same as `.replace()` but affix the queries to the end of the route.
    #[cfg(feature = "query")]
    fn replace_with_query<'a, Q>(
        &self,
        route: impl Into<Cow<'a, str>>,
        query: Q,
    ) -> HistoryResult<(), Q::Error>
    where
        Q: ToQuery;

    /// Same as `.push_with_state()` but affix the queries to the end of the route.
    #[cfg(feature = "query")]
    fn push_with_query_and_state<'a, Q, T>(
        &self,
        route: impl Into<Cow<'a, str>>,
        query: Q,
        state: T,
    ) -> HistoryResult<(), Q::Error>
    where
        Q: ToQuery,
        T: 'static;

    /// Same as `.replace_with_state()` but affix the queries to the end of the route.
    #[cfg(feature = "query")]
    fn replace_with_query_and_state<'a, Q, T>(
        &self,
        route: impl Into<Cow<'a, str>>,
        query: Q,
        state: T,
    ) -> HistoryResult<(), Q::Error>
    where
        Q: ToQuery,
        T: 'static;

    /// Creates a Listener that will be notified when current state changes.
    ///
    /// This method returns a [`HistoryListener`] that will automatically unregister the callback
    /// when dropped.
    fn listen<CB>(&self, callback: CB) -> HistoryListener
    where
        CB: Fn() + 'static;

    /// Returns current [`Location`].
    fn location(&self) -> Location;
}

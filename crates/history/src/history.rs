use std::borrow::Cow;

#[cfg(feature = "serde")]
use serde::Serialize;

#[cfg(feature = "serde")]
use crate::error::HistoryResult;
use crate::listener::HistoryListener;
use crate::location::Location;

/// A trait to provide [`History`] access.
pub trait History: Clone + PartialEq {
    /// The [`Location`] type for current history.
    type Location: Location<History = Self> + 'static;

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
    ///
    /// The implementation of state serialization differs between [`History`] types.
    ///
    /// For [`BrowserHistory`] and [`HashHistory`], state is serialised with [`serde_wasm_bindgen`] where as
    /// [`MemoryHistory`] uses [`Any`](std::any::Any).
    #[cfg(feature = "state")]
    fn push_with_state<'a, T>(&self, route: impl Into<Cow<'a, str>>, state: T) -> HistoryResult<()>
    where
        T: Serialize + 'static;

    /// Replaces the current history entry with provided route and state.
    ///
    /// The implementation of state serialization differs between [`History`] types.
    ///
    /// For [`BrowserHistory`], it uses [`serde_wasm_bindgen`] where as other types uses
    /// [`Any`](std::any::Any).
    #[cfg(feature = "state")]
    fn replace_with_state<'a, T>(
        &self,
        route: impl Into<Cow<'a, str>>,
        state: T,
    ) -> HistoryResult<()>
    where
        T: Serialize + 'static;

    /// Same as `.push()` but affix the queries to the end of the route.
    #[cfg(feature = "query")]
    fn push_with_query<'a, Q>(&self, route: impl Into<Cow<'a, str>>, query: Q) -> HistoryResult<()>
    where
        Q: Serialize;

    /// Same as `.replace()` but affix the queries to the end of the route.
    #[cfg(feature = "query")]
    fn replace_with_query<'a, Q>(
        &self,
        route: impl Into<Cow<'a, str>>,
        query: Q,
    ) -> HistoryResult<()>
    where
        Q: Serialize;

    /// Same as `.push_with_state()` but affix the queries to the end of the route.
    #[cfg(all(feature = "query", feature = "state"))]
    fn push_with_query_and_state<'a, Q, T>(
        &self,
        route: impl Into<Cow<'a, str>>,
        query: Q,
        state: T,
    ) -> HistoryResult<()>
    where
        Q: Serialize,
        T: Serialize + 'static;

    /// Same as `.replace_with_state()` but affix the queries to the end of the route.
    #[cfg(all(feature = "query", feature = "state"))]
    fn replace_with_query_and_state<'a, Q, T>(
        &self,
        route: impl Into<Cow<'a, str>>,
        query: Q,
        state: T,
    ) -> HistoryResult<()>
    where
        Q: Serialize,
        T: Serialize + 'static;

    /// Creates a Listener that will be notified when current state changes.
    ///
    /// This method returns a [`HistoryListener`] that will automatically unregister the callback
    /// when dropped.
    fn listen<CB>(&self, callback: CB) -> HistoryListener
    where
        CB: Fn() + 'static;

    /// Returns the associated [`Location`] of the current history.
    fn location(&self) -> Self::Location;
}

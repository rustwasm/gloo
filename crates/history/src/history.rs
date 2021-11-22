use serde::Serialize;

use crate::error::HistoryResult;
use crate::listener::HistoryListener;
use crate::location::Location;

/// A trait to provide [`History`] access.
pub trait History: Clone + PartialEq {
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
    fn push(&self, route: impl Into<String>);

    /// Replaces the current history entry with provided route and [`None`] state.
    fn replace(&self, route: impl Into<String>);

    /// Pushes a route entry with state.
    ///
    /// The implementation of state serialization differs between [`History`] types.
    ///
    /// For [`BrowserHistory`] and [`HashHistory`], state is serialised with [`serde_wasm_bindgen`] where as
    /// [`MemoryHistory`] uses [`Any`](std::any::Any).
    fn push_with_state<T>(&self, route: impl Into<String>, state: T) -> HistoryResult<()>
    where
        T: Serialize + 'static;

    /// Replaces the current history entry with provided route and state.
    ///
    /// The implementation of state serialization differs between [`History`] types.
    ///
    /// For [`BrowserHistory`], it uses [`serde_wasm_bindgen`] where as other types uses
    /// [`Any`](std::any::Any).
    fn replace_with_state<T>(&self, route: impl Into<String>, state: T) -> HistoryResult<()>
    where
        T: Serialize + 'static;

    /// Same as `.push()` but affix the queries to the end of the route.
    fn push_with_query<Q>(&self, route: impl Into<String>, query: Q) -> HistoryResult<()>
    where
        Q: Serialize;

    /// Same as `.replace()` but affix the queries to the end of the route.
    fn replace_with_query<Q>(&self, route: impl Into<String>, query: Q) -> HistoryResult<()>
    where
        Q: Serialize;

    /// Same as `.push_with_state()` but affix the queries to the end of the route.
    fn push_with_query_and_state<Q, T>(
        &self,
        route: impl Into<String>,
        query: Q,
        state: T,
    ) -> HistoryResult<()>
    where
        Q: Serialize,
        T: Serialize + 'static;

    /// Same as `.replace_with_state()` but affix the queries to the end of the route.
    fn replace_with_query_and_state<Q, T>(
        &self,
        route: impl Into<String>,
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

    /// Returns the State.
    ///
    /// The implementation differs between [`History`] type.
    ///
    /// For [`BrowserHistory`] and [`HashHistory`], state is serialised with [`serde_wasm_bindgen`] where as
    /// [`MemoryHistory`] uses [`Any`](std::any::Any).
    fn state<T>(&self) -> HistoryResult<T>;
}

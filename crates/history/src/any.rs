use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::browser::{BrowserHistory, BrowserLocation};
use crate::error::HistoryResult;
use crate::history::History;
use crate::listener::HistoryListener;
use crate::location::Location;

/// A [`History`] that provides a universial API to the underlying history type.
#[derive(Clone, PartialEq, Debug)]
pub enum AnyHistory {
    /// A Browser History.
    Browser(BrowserHistory),
}

/// The [`Location`] for [`AnyHistory`]
#[derive(Clone, PartialEq, Debug)]
pub enum AnyLocation {
    /// A Browser Location.
    Browser(BrowserLocation),
}

impl History for AnyHistory {
    type Location = AnyLocation;

    fn len(&self) -> usize {
        match self {
            Self::Browser(m) => m.len(),
        }
    }

    fn go(&self, delta: isize) {
        match self {
            Self::Browser(m) => m.go(delta),
        }
    }

    fn push(&self, route: impl Into<String>) {
        match self {
            Self::Browser(m) => m.push(route),
        }
    }

    fn replace(&self, route: impl Into<String>) {
        match self {
            Self::Browser(m) => m.replace(route),
        }
    }

    fn push_with_state<T>(&self, route: impl Into<String>, state: T) -> HistoryResult<()>
    where
        T: Serialize + 'static,
    {
        match self {
            Self::Browser(m) => m.push_with_state(route, state),
        }
    }

    fn replace_with_state<T>(&self, route: impl Into<String>, state: T) -> HistoryResult<()>
    where
        T: Serialize + 'static,
    {
        match self {
            Self::Browser(m) => m.replace_with_state(route, state),
        }
    }

    fn push_with_query<Q>(&self, route: impl Into<String>, query: Q) -> HistoryResult<()>
    where
        Q: Serialize,
    {
        match self {
            Self::Browser(m) => m.push_with_query(route, query),
        }
    }
    fn replace_with_query<Q>(&self, route: impl Into<String>, query: Q) -> HistoryResult<()>
    where
        Q: Serialize,
    {
        match self {
            Self::Browser(m) => m.replace_with_query(route, query),
        }
    }

    fn push_with_query_and_state<Q, T>(
        &self,
        route: impl Into<String>,
        query: Q,
        state: T,
    ) -> HistoryResult<()>
    where
        Q: Serialize,
        T: Serialize + 'static,
    {
        match self {
            Self::Browser(m) => m.push_with_query_and_state(route, query, state),
        }
    }

    fn replace_with_query_and_state<Q, T>(
        &self,
        route: impl Into<String>,
        query: Q,
        state: T,
    ) -> HistoryResult<()>
    where
        Q: Serialize,
        T: Serialize + 'static,
    {
        match self {
            Self::Browser(m) => m.replace_with_query_and_state(route, query, state),
        }
    }

    fn listen<CB>(&self, callback: CB) -> HistoryListener
    where
        CB: Fn() + 'static,
    {
        match self {
            Self::Browser(m) => m.listen(callback),
        }
    }

    fn location(&self) -> Self::Location {
        match self {
            Self::Browser(m) => AnyLocation::Browser(m.location()),
        }
    }
}

impl Location for AnyLocation {
    type History = AnyHistory;

    fn path(&self) -> String {
        match self {
            Self::Browser(m) => m.path(),
        }
    }

    fn search(&self) -> String {
        match self {
            Self::Browser(m) => m.search(),
        }
    }

    fn query<T>(&self) -> HistoryResult<T>
    where
        T: DeserializeOwned,
    {
        match self {
            Self::Browser(m) => m.query(),
        }
    }

    fn hash(&self) -> String {
        match self {
            Self::Browser(m) => m.hash(),
        }
    }

    fn state<T>(&self) -> HistoryResult<T>
    where
        T: DeserializeOwned + 'static,
    {
        match self {
            Self::Browser(m) => m.state(),
        }
    }
}

impl From<BrowserHistory> for AnyHistory {
    fn from(m: BrowserHistory) -> AnyHistory {
        AnyHistory::Browser(m)
    }
}

impl From<BrowserLocation> for AnyLocation {
    fn from(m: BrowserLocation) -> AnyLocation {
        AnyLocation::Browser(m)
    }
}

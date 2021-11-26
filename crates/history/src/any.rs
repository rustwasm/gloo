use std::borrow::Cow;

#[cfg(feature = "serde")]
use serde::de::DeserializeOwned;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::browser::{BrowserHistory, BrowserLocation};
#[cfg(feature = "serde")]
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

    fn push<'a>(&self, route: impl Into<Cow<'a, str>>) {
        match self {
            Self::Browser(m) => m.push(route),
        }
    }

    fn replace<'a>(&self, route: impl Into<Cow<'a, str>>) {
        match self {
            Self::Browser(m) => m.replace(route),
        }
    }

    #[cfg(feature = "state")]
    fn push_with_state<'a, T>(&self, route: impl Into<Cow<'a, str>>, state: T) -> HistoryResult<()>
    where
        T: Serialize + 'static,
    {
        match self {
            Self::Browser(m) => m.push_with_state(route, state),
        }
    }

    #[cfg(feature = "state")]
    fn replace_with_state<'a, T>(
        &self,
        route: impl Into<Cow<'a, str>>,
        state: T,
    ) -> HistoryResult<()>
    where
        T: Serialize + 'static,
    {
        match self {
            Self::Browser(m) => m.replace_with_state(route, state),
        }
    }

    #[cfg(feature = "query")]
    fn push_with_query<'a, Q>(&self, route: impl Into<Cow<'a, str>>, query: Q) -> HistoryResult<()>
    where
        Q: Serialize,
    {
        match self {
            Self::Browser(m) => m.push_with_query(route, query),
        }
    }
    #[cfg(feature = "query")]
    fn replace_with_query<'a, Q>(
        &self,
        route: impl Into<Cow<'a, str>>,
        query: Q,
    ) -> HistoryResult<()>
    where
        Q: Serialize,
    {
        match self {
            Self::Browser(m) => m.replace_with_query(route, query),
        }
    }

    #[cfg(all(feature = "query", feature = "state"))]
    fn push_with_query_and_state<'a, Q, T>(
        &self,
        route: impl Into<Cow<'a, str>>,
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

    #[cfg(all(feature = "query", feature = "state"))]
    fn replace_with_query_and_state<'a, Q, T>(
        &self,
        route: impl Into<Cow<'a, str>>,
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

    #[cfg(feature = "query")]
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

    #[cfg(feature = "state")]
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

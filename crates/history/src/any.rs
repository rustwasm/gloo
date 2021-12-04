use std::borrow::Cow;

#[cfg(feature = "serde")]
use serde::de::DeserializeOwned;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::browser::{BrowserHistory, BrowserLocation};
#[cfg(feature = "serde")]
use crate::error::HistoryResult;
use crate::hash::{HashHistory, HashLocation};
use crate::history::History;
use crate::listener::HistoryListener;
use crate::location::Location;

/// A [`History`] that provides a universial API to the underlying history type.
#[derive(Clone, PartialEq, Debug)]
pub enum AnyHistory {
    /// A Browser History.
    Browser(BrowserHistory),
    /// A Hash History
    Hash(HashHistory),
}

/// The [`Location`] for [`AnyHistory`]
#[derive(Clone, PartialEq, Debug)]
pub enum AnyLocation {
    /// A Browser Location.
    Browser(BrowserLocation),
    /// A Hash Location.
    Hash(HashLocation),
}

impl History for AnyHistory {
    type Location = AnyLocation;

    fn len(&self) -> usize {
        match self {
            Self::Browser(m) => m.len(),
            Self::Hash(m) => m.len(),
        }
    }

    fn go(&self, delta: isize) {
        match self {
            Self::Browser(m) => m.go(delta),
            Self::Hash(m) => m.go(delta),
        }
    }

    fn push<'a>(&self, route: impl Into<Cow<'a, str>>) {
        match self {
            Self::Browser(m) => m.push(route),
            Self::Hash(m) => m.push(route),
        }
    }

    fn replace<'a>(&self, route: impl Into<Cow<'a, str>>) {
        match self {
            Self::Browser(m) => m.replace(route),
            Self::Hash(m) => m.replace(route),
        }
    }

    #[cfg(feature = "state")]
    fn push_with_state<'a, T>(&self, route: impl Into<Cow<'a, str>>, state: T) -> HistoryResult<()>
    where
        T: Serialize + 'static,
    {
        match self {
            Self::Browser(m) => m.push_with_state(route, state),
            Self::Hash(m) => m.push_with_state(route, state),
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
            Self::Hash(m) => m.replace_with_state(route, state),
        }
    }

    #[cfg(feature = "query")]
    fn push_with_query<'a, Q>(&self, route: impl Into<Cow<'a, str>>, query: Q) -> HistoryResult<()>
    where
        Q: Serialize,
    {
        match self {
            Self::Browser(m) => m.push_with_query(route, query),
            Self::Hash(m) => m.push_with_query(route, query),
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
            Self::Hash(m) => m.replace_with_query(route, query),
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
            Self::Hash(m) => m.push_with_query_and_state(route, query, state),
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
            Self::Hash(m) => m.replace_with_query_and_state(route, query, state),
        }
    }

    fn listen<CB>(&self, callback: CB) -> HistoryListener
    where
        CB: Fn() + 'static,
    {
        match self {
            Self::Browser(m) => m.listen(callback),
            Self::Hash(m) => m.listen(callback),
        }
    }

    fn location(&self) -> Self::Location {
        match self {
            Self::Browser(m) => AnyLocation::Browser(m.location()),
            Self::Hash(m) => AnyLocation::Hash(m.location()),
        }
    }
}

impl Location for AnyLocation {
    type History = AnyHistory;

    fn path(&self) -> String {
        match self {
            Self::Browser(m) => m.path(),
            Self::Hash(m) => m.path(),
        }
    }

    fn query_str(&self) -> String {
        match self {
            Self::Browser(m) => m.query_str(),
            Self::Hash(m) => m.query_str(),
        }
    }

    fn hash(&self) -> String {
        match self {
            Self::Browser(m) => m.hash(),
            Self::Hash(m) => m.hash(),
        }
    }

    #[cfg(feature = "state")]
    fn state<T>(&self) -> HistoryResult<T>
    where
        T: DeserializeOwned + 'static,
    {
        match self {
            Self::Browser(m) => m.state(),
            Self::Hash(m) => m.state(),
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

impl From<HashHistory> for AnyHistory {
    fn from(m: HashHistory) -> AnyHistory {
        AnyHistory::Hash(m)
    }
}

impl From<HashLocation> for AnyLocation {
    fn from(m: HashLocation) -> AnyLocation {
        AnyLocation::Hash(m)
    }
}

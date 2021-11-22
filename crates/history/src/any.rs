use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::browser::{BrowserHistory, BrowserLocation};
use crate::error::HistoryResult;
use crate::history::History;
use crate::listener::HistoryListener;
use crate::location::Location;

/// A [`History`] that is always available under a [`Router`](crate::Router).
#[derive(Clone, PartialEq)]
pub enum AnyHistory {
    Browser(BrowserHistory),
}

/// The [`Location`] for [`AnyHistory`]
#[derive(Clone, PartialEq)]
pub enum AnyLocation {
    Browser(BrowserLocation),
}

impl History for AnyHistory {
    type Location = AnyLocation;

    fn len(&self) -> usize {
        let Self::Browser(self_) = self;
        self_.len()
    }

    fn go(&self, delta: isize) {
        let Self::Browser(self_) = self;
        self_.go(delta)
    }

    fn push(&self, route: impl Into<String>) {
        let Self::Browser(self_) = self;
        self_.push(route)
    }

    fn replace(&self, route: impl Into<String>) {
        let Self::Browser(self_) = self;
        self_.replace(route)
    }

    fn push_with_state<T>(&self, route: impl Into<String>, state: T) -> HistoryResult<()>
    where
        T: Serialize + 'static,
    {
        let Self::Browser(self_) = self;
        self_.push_with_state(route, state)
    }

    fn replace_with_state<T>(&self, route: impl Into<String>, state: T) -> HistoryResult<()>
    where
        T: Serialize + 'static,
    {
        let Self::Browser(self_) = self;
        self_.replace_with_state(route, state)
    }

    fn push_with_query<Q>(&self, route: impl Into<String>, query: Q) -> HistoryResult<()>
    where
        Q: Serialize,
    {
        let Self::Browser(self_) = self;
        self_.push_with_query(route, query)
    }
    fn replace_with_query<Q>(&self, route: impl Into<String>, query: Q) -> HistoryResult<()>
    where
        Q: Serialize,
    {
        let Self::Browser(self_) = self;
        self_.replace_with_query(route, query)
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
        let Self::Browser(self_) = self;
        self_.push_with_query_and_state(route, query, state)
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
        let Self::Browser(self_) = self;
        self_.replace_with_query_and_state(route, query, state)
    }

    fn listen<CB>(&self, callback: CB) -> HistoryListener
    where
        CB: Fn() + 'static,
    {
        let Self::Browser(self_) = self;
        self_.listen(callback)
    }

    fn location(&self) -> Self::Location {
        let Self::Browser(self_) = self;
        AnyLocation::Browser(self_.location())
    }

    fn state<T>(&self) -> HistoryResult<T>
    where
        T: DeserializeOwned + 'static,
    {
        let Self::Browser(self_) = self;
        self_.state()
    }
}

impl Location for AnyLocation {
    type History = AnyHistory;

    fn path(&self) -> String {
        let Self::Browser(self_) = self;
        self_.path()
    }

    fn search(&self) -> String {
        let Self::Browser(self_) = self;
        self_.search()
    }

    fn query<T>(&self) -> HistoryResult<T>
    where
        T: DeserializeOwned,
    {
        let Self::Browser(self_) = self;
        self_.query()
    }

    fn hash(&self) -> String {
        let Self::Browser(self_) = self;
        self_.hash()
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

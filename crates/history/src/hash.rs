use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt;

use gloo_utils::window;
#[cfg(feature = "serde")]
use serde::de::DeserializeOwned;
#[cfg(feature = "serde")]
use serde::Serialize;
use wasm_bindgen::{throw_str, UnwrapThrowExt};
use web_sys::Url;

use crate::browser::{BrowserHistory, BrowserLocation};
#[cfg(feature = "serde")]
use crate::error::HistoryResult;
use crate::history::History;
use crate::listener::HistoryListener;
use crate::location::Location;

/// A [`History`] that is implemented with [`web_sys::History`] and stores path in `#`(fragment).
///
/// # Panics
///
/// HashHistory does not support relative paths and will panic if routes are not starting with `/`.
#[derive(Clone, PartialEq)]
pub struct HashHistory {
    inner: BrowserHistory,
}

impl fmt::Debug for HashHistory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HashHistory").finish()
    }
}

impl History for HashHistory {
    type Location = HashLocation;

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn go(&self, delta: isize) {
        self.inner.go(delta)
    }

    fn push<'a>(&self, route: impl Into<Cow<'a, str>>) {
        let route = route.into();

        if !route.starts_with('/') {
            throw_str("You cannot push relative path in hash history.");
        }

        let url = Self::get_url();
        url.set_hash(&route);

        self.inner.push(&url.href());
    }

    fn replace<'a>(&self, route: impl Into<Cow<'a, str>>) {
        let route = route.into();

        if !route.starts_with('/') {
            throw_str("You cannot push relative path in hash history.");
        }

        let url = Self::get_url();
        url.set_hash(&route);

        self.inner.replace(&url.href());
    }

    #[cfg(feature = "state")]
    fn push_with_state<'a, T>(&self, route: impl Into<Cow<'a, str>>, state: T) -> HistoryResult<()>
    where
        T: Serialize + 'static,
    {
        let route = route.into();

        if !route.starts_with('/') {
            throw_str("You cannot push relative path in hash history.");
        }

        let url = Self::get_url();
        url.set_hash(&route);

        self.inner.push_with_state(&url.href(), state)
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
        let route = route.into();

        if !route.starts_with('/') {
            throw_str("You cannot push relative path in hash history.");
        }

        let url = Self::get_url();
        url.set_hash(&route);

        self.inner.replace_with_state(&url.href(), state)
    }

    #[cfg(feature = "query")]
    fn push_with_query<'a, Q>(&self, route: impl Into<Cow<'a, str>>, query: Q) -> HistoryResult<()>
    where
        Q: Serialize,
    {
        let query = serde_urlencoded::to_string(query)?;
        let route = route.into();

        if !route.starts_with('/') {
            throw_str("You cannot push relative path in hash history.");
        }

        let url = Self::get_url();
        url.set_hash(&format!("{}?{}", route, query));

        self.inner.push(&url.href());
        Ok(())
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
        let query = serde_urlencoded::to_string(query)?;
        let route = route.into();

        if !route.starts_with('/') {
            throw_str("You cannot push relative path in hash history.");
        }

        let url = Self::get_url();
        url.set_hash(&format!("{}?{}", route, query));

        self.inner.replace(&url.href());
        Ok(())
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
        let route = route.into();

        if !route.starts_with('/') {
            throw_str("You cannot push relative path in hash history.");
        }

        let url = Self::get_url();

        let query = serde_urlencoded::to_string(query)?;
        url.set_hash(&format!("{}?{}", route, query));

        self.inner.push_with_state(&url.href(), state)
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
        let route = route.into();

        if !route.starts_with('/') {
            throw_str("You cannot push relative path in hash history.");
        }

        let url = Self::get_url();

        let query = serde_urlencoded::to_string(query)?;
        url.set_hash(&format!("{}?{}", route, query));

        self.inner.replace_with_state(&url.href(), state)
    }

    fn listen<CB>(&self, callback: CB) -> HistoryListener
    where
        CB: Fn() + 'static,
    {
        self.inner.listen(callback)
    }

    fn location(&self) -> Self::Location {
        HashLocation::new(self.clone())
    }
}

impl HashHistory {
    /// Creates a new [`HashHistory`]
    pub fn new() -> Self {
        Self::default()
    }

    fn get_url() -> Url {
        let href = window()
            .location()
            .href()
            .expect_throw("Failed to read location href");

        Url::new(&href).expect_throw("current url is not valid.")
    }
}

impl Default for HashHistory {
    fn default() -> Self {
        thread_local! {
            static HASH_HISTORY: RefCell<Option<HashHistory>> = RefCell::default();
        }

        HASH_HISTORY.with(|m| {
            let mut m = m.borrow_mut();

            match *m {
                Some(ref m) => m.clone(),
                None => {
                    let browser_history = BrowserHistory::new();

                    let current_hash = browser_history.location().hash();

                    // Hash needs to start with #/.
                    if current_hash.is_empty() || !current_hash.starts_with("#/") {
                        let url = Self::get_url();
                        url.set_hash("#/");

                        browser_history.replace(url.href());
                    }

                    let history = Self {
                        inner: browser_history,
                    };

                    *m = Some(history.clone());
                    history
                }
            }
        })
    }
}

/// The [`Location`] type for [`HashHistory`].
#[derive(Clone, PartialEq)]
pub struct HashLocation {
    history: HashHistory,
    inner: BrowserLocation,
}

impl fmt::Debug for HashLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HashLocation").finish()
    }
}

impl Location for HashLocation {
    type History = HashHistory;

    fn path(&self) -> String {
        self.location_url().pathname()
    }

    fn search(&self) -> String {
        self.location_url().search()
    }

    #[cfg(feature = "query")]
    fn query<T>(&self) -> HistoryResult<T>
    where
        T: DeserializeOwned,
    {
        let query = self.search();
        serde_urlencoded::from_str(query.strip_prefix('?').unwrap_or("")).map_err(|e| e.into())
    }

    fn hash(&self) -> String {
        self.location_url().hash()
    }

    #[cfg(feature = "state")]
    fn state<T>(&self) -> HistoryResult<T>
    where
        T: DeserializeOwned + 'static,
    {
        self.inner.state()
    }
}

impl HashLocation {
    fn new(history: HashHistory) -> Self {
        Self {
            inner: history.inner.location(),
            history,
        }
    }

    fn location_url(&self) -> Url {
        let hash_url = self.inner.hash().chars().skip(1).collect::<String>();

        assert!(
            hash_url.starts_with('/'),
            "hash-based url cannot be relative path."
        );

        Url::new_with_base(&hash_url, &self.history.inner.location().href())
            .expect_throw("failed to get make url")
    }
}

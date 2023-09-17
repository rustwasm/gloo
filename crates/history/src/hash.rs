use std::{borrow::Cow, fmt};

use gloo_utils::window;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::Url;

use crate::browser::BrowserHistory;
use crate::history::History;
use crate::listener::HistoryListener;
use crate::location::Location;
use crate::utils::{assert_absolute_path, assert_no_query};
#[cfg(feature = "query")]
use crate::{error::HistoryResult, query::ToQuery};

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
    fn len(&self) -> usize {
        self.inner.len()
    }

    fn go(&self, delta: isize) {
        self.inner.go(delta)
    }

    fn push<'a>(&self, route: impl Into<Cow<'a, str>>) {
        let route = route.into();

        assert_absolute_path(&route);
        assert_no_query(&route);

        let url = Self::get_url();
        url.set_hash(&route);

        self.inner.push(url.href());
    }

    fn replace<'a>(&self, route: impl Into<Cow<'a, str>>) {
        let route = route.into();

        assert_absolute_path(&route);
        assert_no_query(&route);

        let url = Self::get_url();
        url.set_hash(&route);

        self.inner.replace(url.href());
    }

    fn push_with_state<'a, T>(&self, route: impl Into<Cow<'a, str>>, state: T)
    where
        T: 'static,
    {
        let route = route.into();

        assert_absolute_path(&route);
        assert_no_query(&route);

        let url = Self::get_url();
        url.set_hash(&route);

        self.inner.push_with_state(url.href(), state)
    }

    fn replace_with_state<'a, T>(&self, route: impl Into<Cow<'a, str>>, state: T)
    where
        T: 'static,
    {
        let route = route.into();

        assert_absolute_path(&route);
        assert_no_query(&route);

        let url = Self::get_url();
        url.set_hash(&route);

        self.inner.replace_with_state(url.href(), state)
    }

    #[cfg(feature = "query")]
    fn push_with_query<'a, Q>(
        &self,
        route: impl Into<Cow<'a, str>>,
        query: Q,
    ) -> HistoryResult<(), Q::Error>
    where
        Q: ToQuery,
    {
        let query = query.to_query()?;
        let route = route.into();

        assert_absolute_path(&route);
        assert_no_query(&route);

        let url = Self::get_url();
        url.set_hash(&format!("{route}?{query}"));

        self.inner.push(url.href());
        Ok(())
    }
    #[cfg(feature = "query")]
    fn replace_with_query<'a, Q>(
        &self,
        route: impl Into<Cow<'a, str>>,
        query: Q,
    ) -> HistoryResult<(), Q::Error>
    where
        Q: ToQuery,
    {
        let query = query.to_query()?;
        let route = route.into();

        assert_absolute_path(&route);
        assert_no_query(&route);

        let url = Self::get_url();
        url.set_hash(&format!("{route}?{query}"));

        self.inner.replace(url.href());
        Ok(())
    }

    #[cfg(feature = "query")]
    fn push_with_query_and_state<'a, Q, T>(
        &self,
        route: impl Into<Cow<'a, str>>,
        query: Q,
        state: T,
    ) -> HistoryResult<(), Q::Error>
    where
        Q: ToQuery,
        T: 'static,
    {
        let route = route.into();

        assert_absolute_path(&route);
        assert_no_query(&route);

        let url = Self::get_url();

        let query = query.to_query()?;
        url.set_hash(&format!("{route}?{query}"));

        self.inner.push_with_state(url.href(), state);

        Ok(())
    }

    #[cfg(feature = "query")]
    fn replace_with_query_and_state<'a, Q, T>(
        &self,
        route: impl Into<Cow<'a, str>>,
        query: Q,
        state: T,
    ) -> HistoryResult<(), Q::Error>
    where
        Q: ToQuery,
        T: 'static,
    {
        let route = route.into();

        assert_absolute_path(&route);
        assert_no_query(&route);

        let url = Self::get_url();

        let query = query.to_query()?;
        url.set_hash(&format!("{route}?{query}"));

        self.inner.replace_with_state(url.href(), state);

        Ok(())
    }

    fn listen<CB>(&self, callback: CB) -> HistoryListener
    where
        CB: Fn() + 'static,
    {
        self.inner.listen(callback)
    }

    fn location(&self) -> Location {
        let inner_loc = self.inner.location();
        // We strip # from hash.
        let hash_url = inner_loc.hash().chars().skip(1).collect::<String>();

        assert_absolute_path(&hash_url);

        let hash_url = Url::new_with_base(
            &hash_url,
            &window()
                .location()
                .href()
                .expect_throw("failed to get location href."),
        )
        .expect_throw("failed to get make url");

        Location {
            path: hash_url.pathname().into(),
            query_str: hash_url.search().into(),
            hash: hash_url.hash().into(),
            id: inner_loc.id,
            state: inner_loc.state,
        }
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
            static HASH_HISTORY: HashHistory = {
                let browser_history = BrowserHistory::new();
                let browser_location = browser_history.location();

                let current_hash = browser_location.hash();

                // Hash needs to start with #/.
                if current_hash.is_empty() || !current_hash.starts_with("#/") {
                    let url = HashHistory::get_url();
                    url.set_hash("#/");

                    browser_history.replace(url.href());
                }

                HashHistory {
                    inner: browser_history,
                }
            };
        }

        HASH_HISTORY.with(|s| s.clone())
    }
}

use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt;
use std::rc::{Rc, Weak};

use gloo_events::EventListener;
use gloo_utils::window;
#[cfg(feature = "serde")]
use serde::de::DeserializeOwned;
#[cfg(feature = "serde")]
use serde::Serialize;
use wasm_bindgen::{JsValue, UnwrapThrowExt};
use web_sys::Url;

#[cfg(feature = "serde")]
use crate::error::HistoryResult;
use crate::history::History;
use crate::listener::HistoryListener;
use crate::location::Location;

type WeakCallback = Weak<dyn Fn()>;

/// A [`History`] that is implemented with [`web_sys::History`] and stores path in `#`(fragment).
#[derive(Clone)]
pub struct HashHistory {
    inner: web_sys::History,
    callbacks: Rc<RefCell<Vec<WeakCallback>>>,
}

impl fmt::Debug for HashHistory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HashHistory").finish()
    }
}

impl PartialEq for HashHistory {
    fn eq(&self, _rhs: &Self) -> bool {
        // All hash histories are created equal.
        true
    }
}

impl History for HashHistory {
    type Location = HashLocation;

    fn len(&self) -> usize {
        self.inner.length().expect_throw("failed to get length.") as usize
    }

    fn go(&self, delta: isize) {
        self.inner
            .go_with_delta(delta as i32)
            .expect_throw("failed to call go.")
    }

    fn push<'a>(&self, route: impl Into<Cow<'a, str>>) {
        let route = route.into();
        assert!(
            route.starts_with('/'),
            "You cannot push relative path in hash history."
        );

        let url = Self::get_url();
        url.set_hash(&route);

        self.inner
            .push_state_with_url(&JsValue::NULL, "", Some(&url.href()))
            .expect_throw("failed to push state.");

        self.notify_callbacks();
    }

    fn replace<'a>(&self, route: impl Into<Cow<'a, str>>) {
        let route = route.into();
        assert!(
            route.starts_with('/'),
            "You cannot push relative path in hash history."
        );

        let url = Self::get_url();
        url.set_hash(&route);

        self.inner
            .replace_state_with_url(&JsValue::NULL, "", Some(&url.href()))
            .expect_throw("failed to replace history.");

        self.notify_callbacks();
    }

    #[cfg(feature = "state")]
    fn push_with_state<'a, T>(&self, route: impl Into<Cow<'a, str>>, state: T) -> HistoryResult<()>
    where
        T: Serialize + 'static,
    {
        let route = route.into();
        assert!(
            route.starts_with('/'),
            "You cannot push relative path in hash history."
        );

        let url = Self::get_url();
        url.set_hash(&route);

        let state = serde_wasm_bindgen::to_value(&state)?;
        self.inner
            .push_state_with_url(&state, "", Some(&url.href()))
            .expect_throw("failed to push state.");

        self.notify_callbacks();
        Ok(())
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
        assert!(
            route.starts_with('/'),
            "You cannot push relative path in hash history."
        );

        let url = Self::get_url();
        url.set_hash(&route);

        let state = serde_wasm_bindgen::to_value(&state)?;
        self.inner
            .replace_state_with_url(&state, "", Some(&url.href()))
            .expect_throw("failed to replace state.");

        self.notify_callbacks();
        Ok(())
    }

    #[cfg(feature = "query")]
    fn push_with_query<'a, Q>(&self, route: impl Into<Cow<'a, str>>, query: Q) -> HistoryResult<()>
    where
        Q: Serialize,
    {
        let query = serde_urlencoded::to_string(query)?;
        let route = route.into();
        assert!(
            route.starts_with('/'),
            "You cannot push relative path in hash history."
        );

        let url = Self::get_url();
        url.set_hash(&format!("{}?{}", route, query));

        self.inner
            .push_state_with_url(&JsValue::NULL, "", Some(&url.href()))
            .expect_throw("failed to push history.");

        self.notify_callbacks();
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
        assert!(
            route.starts_with('/'),
            "You cannot push relative path in hash history."
        );

        let url = Self::get_url();
        url.set_hash(&format!("{}?{}", route, query));

        self.inner
            .replace_state_with_url(&JsValue::NULL, "", Some(&url.href()))
            .expect_throw("failed to replace history.");

        self.notify_callbacks();
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
        assert!(
            route.starts_with('/'),
            "You cannot push relative path in hash history."
        );
        let query = serde_urlencoded::to_string(query)?;
        let state = serde_wasm_bindgen::to_value(&state)?;

        let url = Self::get_url();
        url.set_hash(&format!("{}?{}", route, query));

        self.inner
            .push_state_with_url(&state, "", Some(&url.href()))
            .expect_throw("failed to push history.");

        self.notify_callbacks();
        Ok(())
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
        assert!(
            route.starts_with('/'),
            "You cannot push relative path in hash history."
        );
        let query = serde_urlencoded::to_string(query)?;
        let state = serde_wasm_bindgen::to_value(&state)?;

        let url = Self::get_url();
        url.set_hash(&format!("{}?{}", route, query));

        self.inner
            .replace_state_with_url(&state, "", Some(&url.href()))
            .expect_throw("failed to replace history.");

        self.notify_callbacks();
        Ok(())
    }

    fn listen<CB>(&self, callback: CB) -> HistoryListener
    where
        CB: Fn() + 'static,
    {
        // Callbacks do not receive a copy of [`History`] to prevent reference cycle.
        let cb = Rc::new(callback) as Rc<dyn Fn()>;

        self.callbacks.borrow_mut().push(Rc::downgrade(&cb));

        HistoryListener { _listener: cb }
    }

    fn location(&self) -> Self::Location {
        HashLocation::new(self.clone())
    }
}

impl Default for HashHistory {
    fn default() -> Self {
        // We create browser history only once.
        thread_local! {
            static BROWSER_HISTORY: RefCell<Option<HashHistory>> = RefCell::default();
            static LISTENER: RefCell<Option<EventListener>> = RefCell::default();
        }

        BROWSER_HISTORY.with(|m| {
            let mut m = m.borrow_mut();

            let history = match *m {
                Some(ref m) => m.clone(),
                None => {
                    let window = window();

                    let inner = window
                        .history()
                        .expect_throw("Failed to create hash history. Are you using a browser?");
                    let callbacks = Rc::default();

                    let history = Self { inner, callbacks };

                    {
                        let history = history.clone();

                        // Listens to popstate.
                        LISTENER.with(move |m| {
                            let mut listener = m.borrow_mut();

                            *listener = Some(EventListener::new(&window, "popstate", move |_| {
                                history.notify_callbacks();
                            }));
                        });
                    }

                    history
                }
            };

            *m = Some(history.clone());

            history
        })
    }
}

impl HashHistory {
    /// Creates a new [`HashHistory`]
    pub fn new() -> Self {
        Self::default()
    }

    fn notify_callbacks(&self) {
        let callables = {
            let mut callbacks_ref = self.callbacks.borrow_mut();

            // Any gone weak references are removed when called.
            let (callbacks, callbacks_weak) = callbacks_ref.iter().cloned().fold(
                (Vec::new(), Vec::new()),
                |(mut callbacks, mut callbacks_weak), m| {
                    if let Some(m_strong) = m.clone().upgrade() {
                        callbacks.push(m_strong);
                        callbacks_weak.push(m);
                    }

                    (callbacks, callbacks_weak)
                },
            );

            *callbacks_ref = callbacks_weak;

            callbacks
        };

        for callback in callables {
            callback()
        }
    }

    fn get_url() -> Url {
        let href = window()
            .location()
            .href()
            .expect_throw("Failed to read location href");

        Url::new(&href).expect_throw("current url is not valid.")
    }
}

/// The [`Location`] type for [`HashHistory`].
#[derive(Clone)]
pub struct HashLocation {
    inner: web_sys::Location,
    history: HashHistory,
}

impl fmt::Debug for HashLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HashLocation").finish()
    }
}

impl PartialEq for HashLocation {
    fn eq(&self, rhs: &Self) -> bool {
        self.history == rhs.history
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
        self.inner.hash().expect_throw("failed to get hash.")
    }

    #[cfg(feature = "state")]
    fn state<T>(&self) -> HistoryResult<T>
    where
        T: DeserializeOwned + 'static,
    {
        serde_wasm_bindgen::from_value(
            self.history
                .inner
                .state()
                .expect_throw("failed to read state."),
        )
        .map_err(|e| e.into())
    }
}

impl HashLocation {
    fn new(history: HashHistory) -> Self {
        Self {
            inner: window().location(),
            history,
        }
    }

    fn location_url(&self) -> Url {
        Url::new_with_base(
            &self
                .inner
                .hash()
                .map(|mut m| m.split_off(1))
                .expect_throw("failed to get hash."),
            &self.inner.href().expect_throw("failed to get current url"),
        )
        .expect_throw("failed to get make url")
    }
}

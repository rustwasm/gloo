use std::cell::RefCell;
use std::rc::{Rc, Weak};

use gloo_events::EventListener;
use gloo_utils::window;
use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::{JsValue, UnwrapThrowExt};

use crate::error::HistoryResult;
use crate::history::History;
use crate::listener::HistoryListener;
use crate::location::Location;

type WeakCallback = Weak<dyn Fn()>;

/// A [`History`] that is implemented with [`web_sys::History`] that provides native browser
/// history and state access.
#[derive(Clone)]
pub struct BrowserHistory {
    inner: web_sys::History,
    callbacks: Rc<RefCell<Vec<WeakCallback>>>,
}

impl PartialEq for BrowserHistory {
    fn eq(&self, _rhs: &Self) -> bool {
        // All browser histories are created equal.
        true
    }
}

impl History for BrowserHistory {
    type Location = BrowserLocation;

    fn len(&self) -> usize {
        self.inner.length().expect_throw("failed to get length.") as usize
    }

    fn go(&self, delta: isize) {
        self.inner
            .go_with_delta(delta as i32)
            .expect_throw("failed to call go.")
    }

    fn push(&self, route: impl Into<String>) {
        let url = route.into();
        self.inner
            .push_state_with_url(&JsValue::NULL, "", Some(&url))
            .expect("failed to push state.");

        self.notify_callbacks();
    }

    fn replace(&self, route: impl Into<String>) {
        let url = route.into();
        self.inner
            .replace_state_with_url(&JsValue::NULL, "", Some(&url))
            .expect("failed to replace history.");

        self.notify_callbacks();
    }

    fn push_with_state<T>(&self, route: impl Into<String>, state: T) -> HistoryResult<()>
    where
        T: Serialize + 'static,
    {
        let url = route.into();
        let state = serde_wasm_bindgen::to_value(&state)?;
        self.inner
            .push_state_with_url(&state, "", Some(&url))
            .expect("failed to push state.");

        self.notify_callbacks();
        Ok(())
    }

    fn replace_with_state<T>(&self, route: impl Into<String>, state: T) -> HistoryResult<()>
    where
        T: Serialize + 'static,
    {
        let url = route.into();
        let state = serde_wasm_bindgen::to_value(&state)?;
        self.inner
            .replace_state_with_url(&state, "", Some(&url))
            .expect("failed to replace state.");

        self.notify_callbacks();
        Ok(())
    }

    fn push_with_query<Q>(&self, route: impl Into<String>, query: Q) -> HistoryResult<()>
    where
        Q: Serialize,
    {
        let url = route.into();
        let query = serde_urlencoded::to_string(query)?;
        self.inner
            .push_state_with_url(&JsValue::NULL, "", Some(&format!("{}?{}", url, query)))
            .expect("failed to push history.");

        self.notify_callbacks();
        Ok(())
    }
    fn replace_with_query<Q>(&self, route: impl Into<String>, query: Q) -> HistoryResult<()>
    where
        Q: Serialize,
    {
        let url = route.into();
        let query = serde_urlencoded::to_string(query)?;
        self.inner
            .replace_state_with_url(&JsValue::NULL, "", Some(&format!("{}?{}", url, query)))
            .expect("failed to replace history.");

        self.notify_callbacks();
        Ok(())
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
        let url = route.into();
        let query = serde_urlencoded::to_string(query)?;
        let state = serde_wasm_bindgen::to_value(&state)?;
        self.inner
            .push_state_with_url(&state, "", Some(&format!("{}?{}", url, query)))
            .expect("failed to push history.");

        self.notify_callbacks();
        Ok(())
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
        let url = route.into();
        let query = serde_urlencoded::to_string(query)?;
        let state = serde_wasm_bindgen::to_value(&state)?;
        self.inner
            .replace_state_with_url(&state, "", Some(&format!("{}?{}", url, query)))
            .expect("failed to replace history.");

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
        BrowserLocation::new(self.clone())
    }
}

impl Default for BrowserHistory {
    fn default() -> Self {
        // We create browser history only once.
        thread_local! {
            static BROWSER_HISTORY: RefCell<Option<BrowserHistory>> = RefCell::default();
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
                        .expect_throw("Failed to create browser history. Are you using a browser?");
                    let callbacks = Rc::default();

                    let history = Self { inner, callbacks };

                    let history_clone = history.clone();

                    // Listens to popstate.
                    LISTENER.with(move |m| {
                        let mut listener = m.borrow_mut();

                        *listener = Some(EventListener::new(&window, "popstate", move |_| {
                            history_clone.notify_callbacks();
                        }));
                    });

                    history
                }
            };

            *m = Some(history.clone());

            history
        })
    }
}

impl BrowserHistory {
    /// Creates a new [`BrowserHistory`]
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
}

/// The [`Location`] type for [`BrowserHistory`].
///
/// Most functionality of this type is provided by [`web_sys::Location`].
///
/// This type also provides additional methods that are unique to Browsers and are not available in [`Location`].
///
/// This types is read-only as most setters on `window.location` would cause a reload.
#[derive(Clone)]
pub struct BrowserLocation {
    inner: web_sys::Location,
    history: BrowserHistory,
}

impl PartialEq for BrowserLocation {
    fn eq(&self, rhs: &Self) -> bool {
        self.history == rhs.history
    }
}

impl Location for BrowserLocation {
    type History = BrowserHistory;

    fn path(&self) -> String {
        self.inner
            .pathname()
            .expect_throw("failed to get pathname.")
    }

    fn search(&self) -> String {
        self.inner.search().expect_throw("failed to get search.")
    }

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

impl BrowserLocation {
    fn new(history: BrowserHistory) -> Self {
        Self {
            inner: window().location(),
            history,
        }
    }

    /// Returns the `href` of current [`Location`].
    pub fn href(&self) -> String {
        self.inner.href().expect_throw("failed to get href.")
    }

    /// Returns the `origin` of current [`Location`].
    pub fn origin(&self) -> String {
        self.inner.origin().expect_throw("failed to get origin.")
    }

    /// Returns the `protocol` property of current [`Location`].
    pub fn protocol(&self) -> String {
        self.inner
            .protocol()
            .expect_throw("failed to get protocol.")
    }

    /// Returns the `host` of current [`Location`].
    pub fn host(&self) -> String {
        self.inner.host().expect_throw("failed to get host.")
    }

    /// Returns the `hostname` of current [`Location`].
    pub fn hostname(&self) -> String {
        self.inner
            .hostname()
            .expect_throw("failed to get hostname.")
    }
}

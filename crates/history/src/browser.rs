use std::any::Any;
use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt;
use std::rc::{Rc, Weak};

use gloo_events::EventListener;
use gloo_utils::window;
#[cfg(feature = "query")]
use serde::Serialize;
use wasm_bindgen::{JsValue, UnwrapThrowExt};
use web_sys::Url;

#[cfg(feature = "query")]
use crate::error::HistoryResult;
use crate::history::History;
use crate::listener::HistoryListener;
use crate::location::Location;
use crate::state::{HistoryState, StateMap};

type WeakCallback = Weak<dyn Fn()>;

/// A [`History`] that is implemented with [`web_sys::History`] that provides native browser
/// history and state access.
#[derive(Clone)]
pub struct BrowserHistory {
    inner: web_sys::History,
    states: Rc<RefCell<StateMap>>,
    callbacks: Rc<RefCell<Vec<WeakCallback>>>,
}

impl fmt::Debug for BrowserHistory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BrowserHistory").finish()
    }
}

impl PartialEq for BrowserHistory {
    fn eq(&self, _rhs: &Self) -> bool {
        // All browser histories are created equal.
        true
    }
}

impl History for BrowserHistory {
    fn len(&self) -> usize {
        self.inner.length().expect_throw("failed to get length.") as usize
    }

    fn go(&self, delta: isize) {
        self.inner
            .go_with_delta(delta as i32)
            .expect_throw("failed to call go.")
    }

    fn push<'a>(&self, route: impl Into<Cow<'a, str>>) {
        let url = route.into();
        self.inner
            .push_state_with_url(&Self::create_history_state().1, "", Some(&url))
            .expect_throw("failed to push state.");

        self.notify_callbacks();
    }

    fn replace<'a>(&self, route: impl Into<Cow<'a, str>>) {
        let url = route.into();
        self.inner
            .replace_state_with_url(&Self::create_history_state().1, "", Some(&url))
            .expect_throw("failed to replace history.");

        self.notify_callbacks();
    }

    fn push_with_state<'a, T>(&self, route: impl Into<Cow<'a, str>>, state: T)
    where
        T: 'static,
    {
        let url = route.into();

        let (id, history_state) = Self::create_history_state();

        let mut states = self.states.borrow_mut();
        states.insert(id, Rc::new(state) as Rc<dyn Any>);

        self.inner
            .push_state_with_url(&history_state, "", Some(&url))
            .expect_throw("failed to push state.");

        self.notify_callbacks();
    }

    fn replace_with_state<'a, T>(&self, route: impl Into<Cow<'a, str>>, state: T)
    where
        T: 'static,
    {
        let url = route.into();

        let (id, history_state) = Self::create_history_state();

        let mut states = self.states.borrow_mut();
        states.insert(id, Rc::new(state) as Rc<dyn Any>);
        self.inner
            .replace_state_with_url(&history_state, "", Some(&url))
            .expect_throw("failed to replace state.");

        self.notify_callbacks();
    }

    #[cfg(feature = "query")]
    fn push_with_query<'a, Q>(&self, route: impl Into<Cow<'a, str>>, query: Q) -> HistoryResult<()>
    where
        Q: Serialize,
    {
        let route = route.into();
        let query = serde_urlencoded::to_string(query)?;

        let url = Self::combine_url(&route, &query);

        self.inner
            .push_state_with_url(&Self::create_history_state().1, "", Some(&url))
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
        let route = route.into();
        let query = serde_urlencoded::to_string(query)?;

        let url = Self::combine_url(&route, &query);

        self.inner
            .replace_state_with_url(&Self::create_history_state().1, "", Some(&url))
            .expect_throw("failed to replace history.");

        self.notify_callbacks();
        Ok(())
    }

    #[cfg(all(feature = "query"))]
    fn push_with_query_and_state<'a, Q, T>(
        &self,
        route: impl Into<Cow<'a, str>>,
        query: Q,
        state: T,
    ) -> HistoryResult<()>
    where
        Q: Serialize,
        T: 'static,
    {
        let (id, history_state) = Self::create_history_state();

        let mut states = self.states.borrow_mut();
        states.insert(id, Rc::new(state) as Rc<dyn Any>);

        let route = route.into();
        let query = serde_urlencoded::to_string(query)?;

        let url = Self::combine_url(&route, &query);

        self.inner
            .push_state_with_url(&history_state, "", Some(&url))
            .expect_throw("failed to push history.");

        self.notify_callbacks();
        Ok(())
    }

    #[cfg(all(feature = "query"))]
    fn replace_with_query_and_state<'a, Q, T>(
        &self,
        route: impl Into<Cow<'a, str>>,
        query: Q,
        state: T,
    ) -> HistoryResult<()>
    where
        Q: Serialize,
        T: 'static,
    {
        let route = route.into();
        let query = serde_urlencoded::to_string(query)?;

        let url = Self::combine_url(&route, &query);

        let (id, history_state) = Self::create_history_state();

        let mut states = self.states.borrow_mut();
        states.insert(id, Rc::new(state) as Rc<dyn Any>);
        self.inner
            .replace_state_with_url(&history_state, "", Some(&url))
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

    fn location(&self) -> Location {
        let loc = window().location();

        let history_state = self.inner.state().expect_throw("failed to get state");
        let history_state = serde_wasm_bindgen::from_value::<HistoryState>(history_state).ok();

        let id = history_state.map(|m| m.id());

        let states = self.states.borrow();

        Location {
            path: loc.pathname().expect_throw("failed to get pathname").into(),
            query_str: loc
                .search()
                .expect_throw("failed to get location query.")
                .into(),
            hash: loc
                .hash()
                .expect_throw("failed to get location hash.")
                .into(),
            state: id.and_then(|m| states.get(&m).cloned()),
            id,
        }
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

            match *m {
                Some(ref m) => m.clone(),
                None => {
                    let window = window();

                    let inner = window
                        .history()
                        .expect_throw("Failed to create browser history. Are you using a browser?");
                    let callbacks = Rc::default();

                    let history = Self {
                        inner,
                        callbacks,
                        states: Rc::default(),
                    };

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

                    *m = Some(history.clone());
                    history
                }
            }
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

    fn create_history_state() -> (u32, JsValue) {
        let history_state = HistoryState::new();

        (
            history_state.id(),
            serde_wasm_bindgen::to_value(&history_state)
                .expect_throw("fails to create history state."),
        )
    }

    pub(crate) fn combine_url(route: &str, query: &str) -> String {
        let href = window()
            .location()
            .href()
            .expect_throw("Failed to read location href");

        let url = Url::new_with_base(route, &href).expect_throw("current url is not valid.");

        url.set_search(query);

        url.href()
    }
}

use std::any::Any;
use std::borrow::Cow;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt;
use std::rc::Rc;

use crate::history::History;
use crate::listener::HistoryListener;
use crate::location::Location;
use crate::utils::{
    assert_absolute_path, assert_no_fragment, assert_no_query, get_id, WeakCallback,
};
#[cfg(feature = "query")]
use crate::{error::HistoryResult, query::ToQuery};

/// A History Stack.
#[derive(Debug)]
struct LocationStack {
    prev: Vec<Location>,
    next: VecDeque<Location>,
    current: Location,
}

impl LocationStack {
    fn current(&self) -> Location {
        self.current.clone()
    }

    fn len(&self) -> usize {
        self.prev.len() + self.next.len() + 1
    }

    fn go(&mut self, delta: isize) {
        match delta.cmp(&0) {
            // Go forward.
            Ordering::Greater => {
                for _i in 0..delta {
                    if let Some(mut m) = self.next.pop_front() {
                        std::mem::swap(&mut m, &mut self.current);

                        self.prev.push(m);
                    }
                }
            }
            // Go backward.
            Ordering::Less => {
                for _i in 0..-delta {
                    if let Some(mut m) = self.prev.pop() {
                        std::mem::swap(&mut m, &mut self.current);

                        self.next.push_front(m);
                    }
                }
            }
            // Do nothing.
            Ordering::Equal => {}
        }
    }

    fn push(&mut self, mut location: Location) {
        std::mem::swap(&mut location, &mut self.current);

        self.prev.push(location);
        // When a history is pushed, we clear all forward states.
        self.next.clear();
    }

    fn replace(&mut self, location: Location) {
        self.current = location;
    }
}

impl Default for LocationStack {
    fn default() -> Self {
        Self {
            prev: Vec::new(),
            next: VecDeque::new(),
            current: Location {
                path: "/".to_string().into(),
                query_str: "".to_string().into(),
                hash: "".to_string().into(),
                state: None,
                id: Some(get_id()),
            },
        }
    }
}

/// A [`History`] that is implemented with in memory history stack and is usable in most targets.
///
/// # Panics
///
/// MemoryHistory does not support relative paths and will panic if routes are not starting with `/`.
#[derive(Clone, Default)]
pub struct MemoryHistory {
    inner: Rc<RefCell<LocationStack>>,
    callbacks: Rc<RefCell<Vec<WeakCallback>>>,
}

impl PartialEq for MemoryHistory {
    fn eq(&self, rhs: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &rhs.inner)
    }
}

impl fmt::Debug for MemoryHistory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemoryHistory").finish()
    }
}

impl History for MemoryHistory {
    fn len(&self) -> usize {
        self.inner.borrow().len()
    }

    fn go(&self, delta: isize) {
        self.inner.borrow_mut().go(delta)
    }

    fn push<'a>(&self, route: impl Into<Cow<'a, str>>) {
        let route = route.into();

        assert_absolute_path(&route);
        assert_no_query(&route);
        assert_no_fragment(&route);

        let location = Location {
            path: route.to_string().into(),
            query_str: "".to_string().into(),
            hash: "".to_string().into(),
            state: None,
            id: Some(get_id()),
        };

        self.inner.borrow_mut().push(location);

        self.notify_callbacks();
    }

    fn replace<'a>(&self, route: impl Into<Cow<'a, str>>) {
        let route = route.into();

        assert_absolute_path(&route);
        assert_no_query(&route);
        assert_no_fragment(&route);

        let location = Location {
            path: route.to_string().into(),
            query_str: "".to_string().into(),
            hash: "".to_string().into(),
            state: None,
            id: Some(get_id()),
        };

        self.inner.borrow_mut().replace(location);

        self.notify_callbacks();
    }

    fn push_with_state<'a, T>(&self, route: impl Into<Cow<'a, str>>, state: T)
    where
        T: 'static,
    {
        let route = route.into();

        assert_absolute_path(&route);
        assert_no_query(&route);
        assert_no_fragment(&route);

        let location = Location {
            path: route.to_string().into(),
            query_str: "".to_string().into(),
            hash: "".to_string().into(),
            state: Some(Rc::new(state) as Rc<dyn Any>),
            id: Some(get_id()),
        };

        self.inner.borrow_mut().push(location);

        self.notify_callbacks();
    }

    fn replace_with_state<'a, T>(&self, route: impl Into<Cow<'a, str>>, state: T)
    where
        T: 'static,
    {
        let route = route.into();

        assert_absolute_path(&route);
        assert_no_query(&route);
        assert_no_fragment(&route);

        let location = Location {
            path: route.to_string().into(),
            query_str: "".to_string().into(),
            hash: "".to_string().into(),
            state: Some(Rc::new(state) as Rc<dyn Any>),
            id: Some(get_id()),
        };

        self.inner.borrow_mut().replace(location);

        self.notify_callbacks();
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
        assert_no_fragment(&route);

        let location = Location {
            path: route.to_string().into(),
            query_str: format!("?{query}").into(),
            hash: "".to_string().into(),
            state: None,
            id: Some(get_id()),
        };

        self.inner.borrow_mut().push(location);

        self.notify_callbacks();

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
        assert_no_fragment(&route);

        let location = Location {
            path: route.to_string().into(),
            query_str: format!("?{query}").into(),
            hash: "".to_string().into(),
            state: None,
            id: Some(get_id()),
        };

        self.inner.borrow_mut().replace(location);

        self.notify_callbacks();

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
        let query = query.to_query()?;
        let route = route.into();

        assert_absolute_path(&route);
        assert_no_query(&route);
        assert_no_fragment(&route);

        let location = Location {
            path: route.to_string().into(),
            query_str: format!("?{query}").into(),
            hash: "".to_string().into(),
            state: Some(Rc::new(state) as Rc<dyn Any>),
            id: Some(get_id()),
        };

        self.inner.borrow_mut().push(location);

        self.notify_callbacks();

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
        let query = query.to_query()?;
        let route = route.into();

        assert_absolute_path(&route);
        assert_no_query(&route);
        assert_no_fragment(&route);

        let location = Location {
            path: route.to_string().into(),
            query_str: format!("?{query}").into(),
            hash: "".to_string().into(),
            state: Some(Rc::new(state) as Rc<dyn Any>),
            id: Some(get_id()),
        };

        self.inner.borrow_mut().replace(location);

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
        self.inner.borrow().current()
    }
}

impl MemoryHistory {
    /// Creates a new [`MemoryHistory`] with a default entry of '/'.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`MemoryHistory`] with entries.
    pub fn with_entries<'a>(entries: impl IntoIterator<Item = impl Into<Cow<'a, str>>>) -> Self {
        let self_ = Self::new();

        for (index, entry) in entries.into_iter().enumerate() {
            if index == 0 {
                self_.replace(entry);
            } else {
                self_.push(entry);
            }
        }

        self_
    }

    fn notify_callbacks(&self) {
        crate::utils::notify_callbacks(self.callbacks.clone());
    }
}

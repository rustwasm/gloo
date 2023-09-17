use std::any::Any;
use std::rc::Rc;

#[cfg(feature = "query")]
use crate::{error::HistoryResult, query::FromQuery};

/// A history location.
///
/// This struct provides location information at the time
/// [`History::location`][crate::History::location] is called.
#[derive(Clone, Debug)]
pub struct Location {
    pub(crate) path: Rc<String>,
    pub(crate) query_str: Rc<String>,
    pub(crate) hash: Rc<String>,
    pub(crate) state: Option<Rc<dyn Any>>,
    pub(crate) id: Option<u32>,
}

impl Location {
    /// Returns a unique id of current location.
    ///
    /// Returns [`None`] if current location is not created by `gloo::history`.
    ///
    /// # Warning
    ///
    /// Depending on the situation, the id may or may not be sequential / incremental.
    pub fn id(&self) -> Option<u32> {
        self.id
    }

    /// Returns the `pathname` of current location.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the queries of current URL in [`&str`].
    pub fn query_str(&self) -> &str {
        &self.query_str
    }

    /// Returns the queries of current URL parsed as `T`.
    #[cfg(feature = "query")]
    pub fn query<T>(&self) -> HistoryResult<T::Target, T::Error>
    where
        T: FromQuery,
    {
        let query = self.query_str().strip_prefix('?').unwrap_or("");
        T::from_query(query)
    }

    /// Returns the hash fragment of current URL.
    pub fn hash(&self) -> &str {
        &self.hash
    }

    /// Returns an Rc'ed state of current location.
    ///
    /// Returns [`None`] if state is not created by `gloo::history`, or state fails to downcast.
    pub fn state<T>(&self) -> Option<Rc<T>>
    where
        T: 'static,
    {
        self.state.clone().and_then(|m| m.downcast().ok())
    }
}

impl PartialEq for Location {
    fn eq(&self, rhs: &Self) -> bool {
        if let Some(lhs) = self.id() {
            if let Some(rhs) = rhs.id() {
                return lhs == rhs;
            }
        }
        false
    }
}

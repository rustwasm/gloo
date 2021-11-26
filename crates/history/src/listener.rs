use std::fmt;
use std::rc::Rc;

/// A History Listener to manage callbacks registered on a [`History`].
///
/// This Listener has the same behaviour as the [`EventListener`] from [`gloo`]
/// that the underlying callback will be unregistered when the listener is dropped.
#[must_use]
pub struct HistoryListener {
    pub(crate) _listener: Rc<dyn Fn()>,
}

impl fmt::Debug for HistoryListener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HistoryListener").finish()
    }
}
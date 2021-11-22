use std::rc::Rc;

/// A History Listener to manage callbacks registered on a [`History`].
///
/// This Listener has the same behaviour as the [`EventListener`] from [`gloo`]
/// that the underlying callback will be unregistered when the listener is dropped.
pub struct HistoryListener {
    _listener: Rc<dyn Fn()>,
}

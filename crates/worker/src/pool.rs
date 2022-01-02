use super::*;
use crate::Shared;
use gloo_console as console;
use slab::Slab;

pub(crate) type Last = bool;

/// Type alias to a sharable Slab that owns optional callbacks that emit messages of the type of the specified Worker.
pub(crate) type SharedOutputSlab<W> = Shared<Slab<Option<Callback<<W as Worker>::Output>>>>;

/// The slab contains the callback, the id is used to look up the callback,
/// and the output is the message that will be sent via the callback.
pub(crate) fn locate_callback_and_respond<W: Worker>(
    slab: &SharedOutputSlab<W>,
    id: HandlerId,
    output: W::Output,
) {
    let callback = {
        let slab = slab.borrow();
        match slab.get(id.raw_id()).cloned() {
            Some(callback) => callback,
            None => {
                console::warn!(format!(
                    "Id of handler does not exist in the slab: {}.",
                    id.raw_id()
                ));
                return;
            }
        }
    };
    match callback {
        Some(callback) => (*callback)(output),
        None => console::warn!(format!("The Id of the handler: {}, while present in the slab, is not associated with a callback.", id.raw_id())),
    }
}

/// A newtype around a bridge to indicate that it is distinct from a normal bridge
pub struct Dispatcher<T>(pub(crate) Box<dyn Bridge<T>>);

impl<T> fmt::Debug for Dispatcher<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Dispatcher<_>")
    }
}

impl<T> Deref for Dispatcher<T> {
    type Target = dyn Bridge<T>;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
impl<T> DerefMut for Dispatcher<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

/// This trait allows the creation of a dispatcher to an existing worker that will not send replies when messages are sent.
pub trait Dispatched: Worker + Sized + 'static {
    /// Creates a dispatcher to the worker that will not send messages back.
    ///
    /// # Note
    /// Dispatchers don't have `HandlerId`s and therefore `Worker::handle` will be supplied `None`
    /// for the `id` parameter, and `connected` and `disconnected` will not be called.
    ///
    /// # Important
    /// Because the Workers using Context or Public reaches use the number of existing bridges to
    /// keep track of if the worker itself should exist, creating dispatchers will not guarantee that
    /// an Worker will exist to service requests sent from Dispatchers. You **must** keep at least one
    /// bridge around if you wish to use a dispatcher. If you are using workers in a write-only manner,
    /// then it is suggested that you create a bridge that handles no-op responses as high up in the
    /// component hierarchy as possible - oftentimes the root component for simplicity's sake.
    fn dispatcher() -> Dispatcher<Self>;
}

#[doc(hidden)]
pub trait Dispatchable {}

impl<T> Dispatched for T
where
    T: Worker,
    <T as Worker>::Reach: Discoverer<Worker = T>,
    <T as Worker>::Reach: Dispatchable,
{
    fn dispatcher() -> Dispatcher<T> {
        Dispatcher(Self::Reach::spawn_or_join(None))
    }
}

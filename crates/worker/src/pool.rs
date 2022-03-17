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

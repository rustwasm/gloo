use std::cell::RefCell;
use std::collections::{hash_map::Entry, HashMap, HashSet};
use std::hash::Hash;

/// Thread-local instance used to queue worker messages
pub struct Queue<T: Eq + Hash> {
    loaded_workers: RefCell<HashSet<T>>,
    msg_queue: RefCell<HashMap<T, Vec<Vec<u8>>>>,
}

impl<T: Eq + Hash> Queue<T> {
    pub fn new() -> Queue<T> {
        Queue {
            loaded_workers: RefCell::new(HashSet::new()),
            msg_queue: RefCell::new(HashMap::new()),
        }
    }

    #[inline]
    pub fn remove_msg_queue(&self, id: &T) -> Option<Vec<Vec<u8>>> {
        self.msg_queue.borrow_mut().remove(id)
    }

    #[inline]
    pub fn insert_loaded_worker(&self, id: T) {
        self.loaded_workers.borrow_mut().insert(id);
    }

    #[inline]
    pub fn is_worker_loaded(&self, id: &T) -> bool {
        self.loaded_workers.borrow().contains(id)
    }

    pub fn add_msg_to_queue(&self, msg: Vec<u8>, id: T) {
        let mut queue = self.msg_queue.borrow_mut();
        match queue.entry(id) {
            Entry::Vacant(record) => {
                record.insert(vec![msg]);
            }
            Entry::Occupied(ref mut record) => {
                record.get_mut().push(msg);
            }
        }
    }

    /// This is called by a worker's `Drop` implementation in order to remove the worker from the list
    /// of loaded workers.
    pub fn remove_worker(&self, id: &T) {
        self.loaded_workers.borrow_mut().remove(id);
        self.msg_queue.borrow_mut().remove(id);
    }
}

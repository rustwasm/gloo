use wasm_bindgen::{JsValue, UnwrapThrowExt};

/// A wrapper around JS Iterator so it can be consumed from Rust.
///
/// This type implements [`Iterator`] trait and will keep yielding [`JsValue`]
/// until the underlying [`js_sys::Iterator`] is exuasted.
///
/// This type is called `UncheckedIter` because it does no checking for
/// the underlying type of the [`js_sys::Iterator`] and yields [`JsValue`]s.
///
/// # Example
///
/// ```rust
/// use gloo_utils::iter::UncheckedIter;
/// use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
///
/// # fn no_run() {
/// let map = js_sys::Map::new();
/// map.set(&JsValue::from("one"), &JsValue::from(1_f64));
///
/// let mut iter = UncheckedIter::from(map.entries()).map(|js_value| {
///     let array: js_sys::Array = js_value.unchecked_into();
///     (
///         array.get(0).as_string().unwrap_throw(),
///         array.get(1).as_f64().unwrap_throw(),
///     )
/// });
///
/// assert_eq!(iter.next(), Some((String::from("one"), 1_f64)));
/// assert_eq!(iter.next(), None);
/// # }
/// ```
pub struct UncheckedIter(js_sys::Iterator);

impl UncheckedIter {
    /// Obtain the raw [`js_sys::Iterator`]
    pub fn into_raw(self) -> js_sys::Iterator {
        self.0
    }
}

impl From<js_sys::Iterator> for UncheckedIter {
    fn from(iter: js_sys::Iterator) -> Self {
        Self(iter)
    }
}

impl Iterator for UncheckedIter {
    type Item = JsValue;

    fn next(&mut self) -> Option<Self::Item> {
        // we don't check for errors. Only use this type on things we know conform to the iterator
        // interface.
        let next = self.0.next().unwrap_throw();
        if next.done() {
            None
        } else {
            Some(next.value())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn it_works() {
        let map = js_sys::Map::new();
        macro_rules! map_set {
            ($key:expr => $value:expr) => {
                map.set(&JsValue::from($key), &JsValue::from($value));
            };
        }

        map_set!("one" => 1_f64);
        map_set!("two" => 2_f64);
        map_set!("three" => 3_f64);

        let mut iter = UncheckedIter::from(map.entries()).map(|js_value| {
            let array = js_sys::Array::from(&js_value);
            let array = array.to_vec();
            (
                array[0].as_string().expect_throw("not string"),
                array[1].as_f64().expect_throw("not f64"),
            )
        });

        assert_eq!(iter.next(), Some((String::from("one"), 1_f64)));
        assert_eq!(iter.next(), Some((String::from("two"), 2_f64)));
        assert_eq!(iter.next(), Some((String::from("three"), 3_f64)));
        assert_eq!(iter.next(), None);
    }
}

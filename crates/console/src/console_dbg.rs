/// A macro similar to [`dbg!`] that logs [`JsValue`][wasm_bindgen::JsValue]s to console.
///
/// See the [stdlib documentation][std::dbg] to learn more. This macro calls `console.log`
/// instead of `eprintln!` for `JsValue`s. The formatting is done by the browser. If you want
/// [`Debug`][std::fmt::Debug] implementation to be used instead, consider using [`console_dbg`]
#[macro_export]
macro_rules! console {
    () => {
        $crate::log!(
            ::std::format!("%c[{}:{}] ", ::std::file!(), ::std::line!()),
            "font-weight: bold"
        );
    };
    ($val:expr $(,)?) => {
        {
            let v = $val;
            $crate::__console_inner!(v $val)
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::console!($val)),+,)
    };
}

/// A macro similar to [`dbg!`] to log to browser console.
///
/// See the [stdlib documentation][std::dbg] to learn more. This macro calls `console.log`
/// instead of `eprintln!`. This macro passing the values to [`console`] after formatting them using
/// the [`Debug`][std::fmt::Debug] implementation.
#[macro_export]
macro_rules! console_dbg {
    () => {
        $crate::console!()
    };
    ($val:expr $(,)?) => {
        {
            let v: $crate::__macro::JsValue = ::std::format!("{:?}", $val).into();
            $crate::__console_inner!(v $val)
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::console_dbg!($val)),+,)
    };
}

/// This is an implementation detail and *should not* be called directly!
#[doc(hidden)]
#[macro_export]
macro_rules! __console_inner {
    ($js_value:ident $val:expr) => {{
        $crate::log!(
            ::std::format!("%c[{}:{}] ", ::std::file!(), ::std::line!()),
            "font-weight: bold",
            ::std::format!("{} = ", ::std::stringify!($val)),
            &$js_value
        );
        $js_value
    }};
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    //! These exist to ensure code compiles
    use wasm_bindgen::JsValue;

    fn console_works() {
        console!();
        {
            let js_value = JsValue::from("test");
            console!(js_value);
        }
        {
            let js_value_1 = JsValue::from("test 1");
            let js_value_2 = JsValue::from("test 2");
            console!(js_value_1, js_value_2);
        }
    }

    fn console_dbg_works() {
        #[derive(Debug)]
        struct Value(&'static str);

        console_dbg!();
        {
            let value = Value("test");
            console_dbg!(value);
        }
        {
            let value_1 = Value("test 1");
            let value_2 = Value("test 2");
            console_dbg!(value_1, value_2);
        }
    }
}

/// A macro similar to [`dbg!`].
///
/// See the [stdlib documentation][std::dbg] to learn more. This macro calls `console.log`
/// instead of `eprintln!` for `JsValue`s. The formatting is done by the browser. If you want
/// [`Debug`][std::fmt::Debug] implementation to be used instead, consider using [`console_dbg`]
#[macro_export]
macro_rules! console {
    () => {
        $crate::log!(::std::format!("[{}:{}]", ::std::file!(), ::std::line!()));
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::log!(::std::format!("[{}:{}] {} =", ::std::file!(), ::std::line!(), ::std::stringify!($val)), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::console!($val)),+,)
    };
}

/// A macro similar to [`dbg!`].
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
        match $val {
            v => {
                $crate::console!(::std::format!("{:?}", v));
                v
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::console_dbg!($val)),+,)
    };
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

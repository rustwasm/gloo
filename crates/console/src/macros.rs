#[macro_export]
macro_rules! clear {
    () => {
       $crate::externs::clear();
    }
}

#[macro_export]
macro_rules! assert {
    ($assertion:expr, $($arg:expr),+) => {
       $crate::externs::assert($assertion, ::std::boxed::Box::from([$($crate::__macro::JsValue::from($arg),)+]));
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:expr),+) => {
       $crate::externs::debug(::std::boxed::Box::from([$($crate::__macro::JsValue::from($arg),)+]));
    }
}

#[macro_export]
macro_rules! dir {
    ($($arg:expr),+) => {
       $crate::externs::dir($crate::__macro::JsValue::from($arg));
    }
}

#[macro_export]
macro_rules! dirxml {
    ($($arg:expr),+) => {
       $crate::externs::dirxml($crate::__macro::JsValue::from($arg));
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:expr),+) => {
       $crate::externs::error(::std::boxed::Box::from([$($crate::__macro::JsValue::from($arg),)+]));
    }
}

#[macro_export]
macro_rules! info {
    ($($arg:expr),+) => {
       $crate::externs::info(::std::boxed::Box::from([$($crate::__macro::JsValue::from($arg),)+]));
    }
}

/// We use serde to serialize the passed data object into JsValue.
#[macro_export]
macro_rules! table {
    ($data:expr) => {
       $crate::externs::table_with_data($crate::__macro::JsValue::from($data));
    };
    ($data:expr, $columns:expr) => {
        $crate::__macro::table_with_data_and_columns($data, $columns);
    }
}


#[macro_export]
macro_rules! log {
    ($($arg:expr),+) => {
       $crate::externs::log(::std::boxed::Box::from([$($crate::__macro::JsValue::from($arg),)+]));
    }
}

#[macro_export]
macro_rules! trace {
    ($($arg:expr),+) => {
       $crate::externs::trace(::std::boxed::Box::from([$($crate::__macro::JsValue::from($arg),)+]));
    }
}

#[macro_export]
macro_rules! warn {
    ($($arg:expr),+) => {
       $crate::externs::warn(::std::boxed::Box::from([$($crate::__macro::JsValue::from($arg),)+]));
    }
}

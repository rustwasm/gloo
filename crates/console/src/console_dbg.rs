/// A macro similar to [`dbg!`].
///
/// See the [stdlib documentation][std::dbg] to learn more. This macro calls `console.log`
/// instead of `eprintln!`
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
        ($($crate::dbg!($val)),+,)
    };
}

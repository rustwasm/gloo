/// A helper macro to generate `from_jsvalue` impls for errors.
///
/// Expects one variant to look like `Unexpected(String)`.
macro_rules! error_from_jsvalue {
    ($name:ident {$($str:expr => $variant:ident),* $(,)? }) => {
        impl From<::web_sys::DomException> for $name {
            fn from(error: ::web_sys::DomException) -> Self {
                let name = error.name();
                match name.as_str() {
                    $($str => Self::$variant,)*
                    _ => Self::Unexpected(error.message()),
                }
            }
        }
        impl From<::wasm_bindgen::JsValue> for $name {
            fn from(raw: ::wasm_bindgen::JsValue) -> Self {
                let error = match ::wasm_bindgen::JsCast::dyn_into::<::web_sys::DomException>(raw) {
                    Ok(error) => error,
                    Err(_) => return Self::Unexpected("".into()),
                };
                Self::from(error)
            }
        }
    };
}

use super::utils::options;

pub struct NumberFormat {
    js: js_sys::Intl::NumberFormat,
}

// TODO: Add a `new` method that takes a strongly types locale. Maybe an enum.
impl NumberFormat {
    /// New NumberFormat instance from locale &str.
    pub fn new_str(locale: &str, options: Options) -> Self {
        let locales = js_sys::Array::of1(&wasm_bindgen::JsValue::from_str(locale));
        Self::new_js_array(locales, options)
    }
    /// New NumberFormat instance from js_sys::Array of locales.
    pub fn new_js_array(locales: js_sys::Array, options: Options) -> Self {
        let options = options.to_js_object();
        let js = js_sys::Intl::NumberFormat::new(&locales, &options);
        Self { js }
    }
    /// Corresponds to https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/NumberFormat/format
    pub fn format<T>(&self, number: T) -> String
    where
        T: Into<f64>,
    {
        let number: f64 = number.into();
        self.js
            .format()
            .call1(&self.js, &number.into())
            .unwrap()
            .as_string()
            .unwrap()
    }
}

/// Corresponds to the options in https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/NumberFormat/NumberFormat
// TODO: add all options here.
#[derive(Clone, Default)]
pub struct Options {
    pub style: Option<Style>,
    pub currency_display: Option<CurrencyDisplay>,
    pub currency_sign: Option<CurrencySign>,
}
impl Options {
    // TODO: should be a derive macro?
    fn to_js_object(&self) -> js_sys::Object {
        let output = js_sys::Object::new();
        if let Some(year) = &self.style {
            pro(&output, "style", year.string_rep());
        }
        if let Some(month) = &self.currency_display {
            pro(&output, "currencyDisplay", month.string_rep());
        }
        if let Some(day) = &self.currency_sign {
            pro(&output, "currencySign", day.string_rep());
        }
        output
    }
}

fn pro(obj: &js_sys::Object, key: &str, value: &str) {
    js_sys::Reflect::set(
        &obj,
        &wasm_bindgen::JsValue::from_str(key),
        &wasm_bindgen::JsValue::from_str(value),
    )
    .unwrap();
}

options!(Style, {
    Decimal: "decimal",
    Currency: "currency",
    Percent: "percent",
    Unit: "unit",
});
options!(CurrencyDisplay, {
    Code: "code",
    Symbol: "symbol",
    NarrowSymbol: "narrowSymbol",
    Name: "name",
});
options!(CurrencySign, {
    Standard: "standard",
    Accounting: "accounting",
});

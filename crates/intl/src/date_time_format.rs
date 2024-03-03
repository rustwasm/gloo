use super::utils::options;

pub struct DateTimeFormat {
    js: js_sys::Intl::DateTimeFormat,
}

// TODO: Add a `new` method that takes a strongly types locale. Maybe an enum.
impl DateTimeFormat {
    /// New DateTimeFormat instance from locale &str.
    pub fn new_str(locale: &str, options: Options) -> Self {
        let locales = js_sys::Array::of1(&wasm_bindgen::JsValue::from_str(locale));
        Self::new_js_array(locales, options)
    }
    /// New DateTimeFormat instance from js_sys::Array of locales.
    pub fn new_js_array(locales: js_sys::Array, options: Options) -> Self {
        let options = options.to_js_object();
        let js = js_sys::Intl::DateTimeFormat::new(&locales, &options);
        Self { js }
    }
    /// Corresponds to https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/DateTimeFormat/format
    pub fn format(&self, date: &js_sys::Date) -> String {
        self.js
            .format()
            .call1(&self.js, &date.into())
            .unwrap()
            .as_string()
            .unwrap()
    }
}

/// Corresponds to the options in https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/DateTimeFormat/DateTimeFormat
// TODO: add all options here.
#[derive(Clone, Default)]
pub struct Options {
    pub year: Option<Year>,
    pub month: Option<Month>,
    pub day: Option<Day>,
    pub hour: Option<Hour>,
    pub minute: Option<Minute>,
    pub second: Option<Second>,
}
impl Options {
    // TODO: should be a derive macro?
    fn to_js_object(&self) -> js_sys::Object {
        let output = js_sys::Object::new();
        if let Some(year) = &self.year {
            pro(&output, "year", year.string_rep());
        }
        if let Some(month) = &self.month {
            pro(&output, "month", month.string_rep());
        }
        if let Some(day) = &self.day {
            pro(&output, "day", day.string_rep());
        }
        if let Some(hour) = &self.hour {
            pro(&output, "hour", hour.string_rep());
        }
        if let Some(minute) = &self.minute {
            pro(&output, "minute", minute.string_rep());
        }
        if let Some(second) = &self.second {
            pro(&output, "second", second.string_rep());
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

options!(Year, {
    Numeric: "numeric",
    Digit2: "2-digit",
});
options!(Month, {
    Numeric: "numeric",
    Digit2: "2-digit",
    Long: "long",
    Short: "short",
    Narrow: "narrow",
});
options!(Day, {
    Numeric: "numeric",
    Digit2: "2-digit",
});
options!(Hour, {
    Numeric: "numeric",
    Digit2: "2-digit",
});
options!(Minute, {
    Numeric: "numeric",
    Digit2: "2-digit",
});
options!(Second, {
    Numeric: "numeric",
    Digit2: "2-digit",
});

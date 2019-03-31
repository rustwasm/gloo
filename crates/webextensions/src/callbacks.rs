//! Low-level bindings for WebExtensions API
//!
//! [MDN Documentation](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions)

use std::marker::PhantomData;

use js_sys::{Function, JsString, Object, Promise};
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
extern "C" {
    static browser: Browser;
}

#[wasm_bindgen]
extern "C" {
    pub type Browser;

    /// Schedule code to run at a specific time in the future.
    ///
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/alarms)
    #[wasm_bindgen(method, getter)]
    pub fn alarms(this: &Browser) -> Alarms;

    /// A page action is a clickable icon inside the browser's address bar.
    ///
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/pageAction)
    #[wasm_bindgen(method, getter, js_name = pageAction)]
    pub fn page_action(this: &Browser) -> PageAction;

    /// Interact with the browser's tab system.
    ///
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/tabs)
    #[wasm_bindgen(method, getter)]
    pub fn tabs(this: &Browser) -> Tabs;
}

/// Getter for the `Alarms` object.
pub fn alarms() -> Alarms {
    browser.alarms()
}

/// You can use this to specify when the alarm will initially fire, either as
/// an absolute value (`when`), or as a delay from the time the alarm is set
/// (`delay_in_minutes`). To make the alarm recur, specify `period_in_minutes`.
#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct AlarmInfo {
    /// The time the alarm will fire first, given as milliseconds since the epoch.
    /// If you specify `when`, don't specify `delay_in_minutes`.
    pub when: Option<u64>,

    /// The time the alarm will fire first, given as minutes from the time the
    /// alarm is set.
    /// If you specify `delay_in_minutes`, don't specify `when`.
    pub delay_in_minutes: Option<f64>,

    /// If this is specified, the alarm will fire again every `period_in_minutes`
    /// after its initial firing. If you specify this value you may omit both
    /// when and `delay_in_minutes`, and the alarm will then fire initially after
    /// periodInMinutes. If `period_in_minutes` is not specified, the alarm will
    /// only fire once.
    pub period_in_minutes: Option<f64>,
}

#[wasm_bindgen]
extern "C" {
    pub type Alarms;

    /// Cancels an alarm, given its name.
    ///
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/alarms/clear)
    #[wasm_bindgen(method, js_name = clear)]
    pub fn clear_with_name(this: &Alarms, name: &JsString) -> Promise;

    /// Cancels all active alarms.
    ///
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/alarms/clearAll)
    #[wasm_bindgen(method, js_name = clearAll)]
    pub fn clear_all(this: &Alarms) -> Promise;

    /// Creates a new alarm for the current browser session. An alarm may fire
    /// once or multiple times. An alarm is cleared after it fires for the last
    /// time.
    ///
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/alarms/create)
    #[wasm_bindgen(method, js_name = create)]
    pub fn create_with_info(this: &Alarms, alarm_info: &Object);

    /// Creates a new alarm for the current browser session. An alarm may fire
    /// once or multiple times. An alarm is cleared after it fires for the last
    /// time.
    ///
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/alarms/create)
    #[wasm_bindgen(method, js_name = create)]
    pub fn create_with_name_and_info(this: &Alarms, name: &str, alarm_info: &Object);

    #[wasm_bindgen(method, getter, js_name = onAlarm)]
    fn raw_on_alarm(this: &Alarms) -> RawEvents;
}

impl Alarms {
    /// Fired when any alarm set by the extension goes off.
    ///
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/alarms/onAlarm)
    pub fn on_alarm(&self) -> Events<Alarm> {
        Events::new(self.raw_on_alarm())
    }
}

#[wasm_bindgen]
extern "C" {
    /// Information about a single alarm.
    ///
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/alarms/Alarm)
    pub type Alarm;

    /// Name of this alarm. This is the name that was passed into the
    /// `Alarms::create()` call that created this alarm.
    #[wasm_bindgen(method, getter)]
    pub fn name(this: &Alarm) -> String;

    /// Time at which the alarm is scheduled to fire next, in milliseconds since
    /// the epoch.
    #[wasm_bindgen(method, getter, js_name = scheduledTime)]
    pub fn scheduled_time(this: &Alarm) -> u64;

    /// If this is not `None`, then the alarm is periodic, and this represents
    /// its period in minutes.
    #[wasm_bindgen(method, getter, js_name = periodInMinutes)]
    pub fn period_in_minutes(this: &Alarm) -> Option<u64>;
}

impl Alarm {
    /// Creates a new alarm for the current browser session. An alarm may fire
    /// once or multiple times. An alarm is cleared after it fires for the last
    /// time.
    ///
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/alarms/create)
    pub fn create_with_name_and_info(name: &str, info: &AlarmInfo) {
        alarms()
            .create_with_name_and_info(name, &JsValue::from_serde(info).unwrap().unchecked_ref());
    }
}

#[wasm_bindgen]
extern "C" {
    type RawEvents;

    #[wasm_bindgen(method, js_name = addListener)]
    fn add_listener(this: &RawEvents, listener: &Function);

    #[wasm_bindgen(method, js_name = removeListener)]
    fn remove_listener(this: &RawEvents, listener: &Function);

    #[wasm_bindgen(method, js_name = hasListener)]
    fn has_listener(this: &RawEvents, listener: &Function);
}

///
#[allow(missing_debug_implementations)]
pub struct Events<T> {
    inner: RawEvents,
    _marker: PhantomData<T>,
}

impl<T> Events<T> {
    fn new(inner: RawEvents) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }

    /// Adds a listener to this event.
    pub fn add_listener(&self, listener: &Closure<dyn FnMut(T)>) {
        self.inner.add_listener(listener.as_ref().unchecked_ref())
    }

    /// Stop listening to this event.
    pub fn remove_listener(&self, listener: &Closure<dyn FnMut(T)>) {
        self.inner
            .remove_listener(listener.as_ref().unchecked_ref())
    }

    /// Check whether listener is registered for this event.
    pub fn has_listener(&self, listener: &Closure<dyn FnMut(T)>) {
        self.inner.has_listener(listener.as_ref().unchecked_ref())
    }
}

/// Getter for the `Tabs` object.
pub fn page_action() -> PageAction {
    browser.page_action()
}

#[wasm_bindgen]
extern "C" {
    pub type PageAction;

    /// Shows the page action for a given tab. The page action is shown whenever
    /// the given tab is the active tab.
    ///
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/pageAction/show)
    #[wasm_bindgen(method)]
    pub fn show(this: &PageAction, tabId: i32);
}

/// Getter for the `Tabs` object.
pub fn tabs() -> Tabs {
    browser.tabs()
}

#[wasm_bindgen]
extern "C" {
    pub type Tabs;

    /// Gets all tabs that have the specified properties, or all tabs if no properties are
    /// specified.
    ///
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/tabs/query)
    #[wasm_bindgen(method)]
    pub fn query(this: &Tabs, query_info: &Object) -> Promise;
}

#[wasm_bindgen]
extern "C" {
    pub type Tab;

    /// The tab's ID. Tab IDs are unique within a browser session.
    #[wasm_bindgen(method, getter)]
    pub fn id(this: &Tab) -> i32;
}

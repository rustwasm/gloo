//! Low-level bindings for WebExtensions API
//!
//! [MDN Documentation](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions)

use js_sys::{JsString, Object, Promise};
use wasm_bindgen::prelude::*;

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
    pub fn create_with_name_and_info(this: &Alarms, name: &JsString, alarm_info: &Object);

    /// Fired when any alarm set by the extension goes off.
    ///
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/alarms/onAlarm)
    #[wasm_bindgen(method, getter, js_name = onAlarm)]
    pub fn on_alarm(this: &Alarms) -> OnAlarm;
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

#[wasm_bindgen]
extern "C" {
    pub type OnAlarm;

    /// Adds a listener to this event.
    #[wasm_bindgen(method, js_name = addListener)]
    pub fn add_listener(this: &OnAlarm, listener: &Closure<FnMut(Alarm)>);
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

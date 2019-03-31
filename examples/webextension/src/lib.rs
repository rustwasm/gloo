use gloo_webextensions::callbacks::{alarms, Alarm, AlarmInfo};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

const DELAY_IN_MINUTES: f64 = 0.1; // 6 seconds

#[wasm_bindgen(start)]
pub fn start() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let alarm_btn: web_sys::HtmlButtonElement = document
        .get_element_by_id("alarm-schedule")
        .expect("element #alarm-schedule should exist")
        .dyn_into()
        .unwrap();

    let closure = Closure::wrap(Box::new(|| {
        let alarm_name_input: web_sys::HtmlInputElement = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("alarm-name")
            .expect("element #alarm-name should exist")
            .dyn_into()
            .unwrap();

        Alarm::create_with_name_and_info(
            &alarm_name_input.value(),
            &AlarmInfo {
                delay_in_minutes: Some(DELAY_IN_MINUTES),
                ..Default::default()
            },
        );

        let listener = Closure::wrap(Box::new(on_alarm) as Box<FnMut(Alarm)>);
        alarms().on_alarm().add_listener(&listener);
        listener.forget();
    }) as Box<FnMut()>);
    alarm_btn.set_onclick(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
}

fn on_alarm(alarm: Alarm) {
    web_sys::window()
        .unwrap()
        .alert_with_message(&format!("Alarm {} fired!", alarm.name()))
        .unwrap();
}

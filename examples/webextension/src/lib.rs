use gloo_webextensions::callbacks::{alarms, Alarm};
use js_sys::{Object, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

const DELAY: f64 = 0.1;

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

        let info = Object::new();
        Reflect::set(&info, &"delayInMinutes".into(), &DELAY.into()).unwrap();
        alarms().create_with_name_and_info(&alarm_name_input.value().into(), &info);

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

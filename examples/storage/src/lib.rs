use dominator::{events, html, text, text_signal, with_node, Dom};
use futures_signals::{
    signal::{Mutable, SignalExt},
    signal_vec::{MutableVec, SignalVecExt},
};
use gloo::storage::indexed_db as idb;
use serde::{Deserialize, Serialize};
use smartstring::alias::String as SmartString;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

type ArcStr = Arc<str>;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    let app = App::new();
    let document = web_sys::window().unwrap_throw().document().unwrap_throw();

    let el = document.get_element_by_id("app").unwrap_throw();

    // render the date, then set it to re-render every second.
    spawn_local(use_db(el));
}

enum App {
    Loading,
    Loaded(AppState),
    Error(String),
}

impl App {
    fn new() -> Mutable<Self> {
        let this = Mutable::new(App::Loading);
        spawn_local({
            let this = this.clone();
            async move {
                let state = AppState::new().await;
                this.set(App::Loaded(state))
            }
        });
        this
    }

    fn render_mutable(this: Mutable<Self>) -> Dom {
        html!("div", {
            .attr("id", "app")
            .child_signal(this.signal_ref(|app| Some(App::render(app))))
        })
    }

    fn render(&self) -> Dom {
        match self {
            App::Loading => text("Loading"),
            App::Loaded(state) => state.render(),
            App::Error(msg) => text(&format!("Error: {}", msg)),
        }
    }
}

struct AppState {
    db: idb::Database,
    user: Person,
    people: MutableVec<Person>,
    new_person: Mutable<NewPerson>,
}

impl AppState {
    async fn new() -> Self {
        let db = idb::Database::open(
            "mydb",
            1,
            |db| {
                let store = db
                    .create_object_store("people")
                    .auto_increment(true)
                    .key_path("id")
                    .build()
                    .unwrap();
                let _ = store.add(&NewPerson::new("Joe", "Bloggs"), None, true);
            },
            true,
        )
        .await
        .unwrap();
        let trans = db.transaction_readonly(&["people"]).unwrap();
        let people_store = trans.object_store("people").unwrap();
        let user = people_store.get(1.).await.unwrap().unwrap();
        let people = people_store.get_all().await.unwrap();
        AppState {
            db,
            user,
            people,
            new_person: Mutable::new(NewPerson::default()),
        }
    }

    fn render(&self) -> Dom {
        html!("div", {
            .child(render_people(&self.people))
            .child(NewPerson::render_form(self.new_person.clone()))
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Person {
    id: u64,
    first_name: SmartString,
    last_name: SmartString,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct NewPerson {
    first_name: SmartString,
    last_name: SmartString,
}

impl NewPerson {
    fn new(first_name: &str, last_name: &str) -> Self {
        Self {
            first_name: first_name.into(),
            last_name: last_name.into(),
        }
    }

    fn render_form(this: Mutable<NewPerson>) -> Dom {
        html!("label", {
            .children(&mut [
                text("first name ("),
                text_signal(this.signal_cloned().map(|p| p.first_name)),
                text(") "),
                html!("input" => web_sys::HtmlInputElement, {
                    .with_node!(input => {
                        .attr("type", "text")
                        .attr_signal("value", this.signal_cloned().map(|person| person.first_name))
                        .event({
                            let this = this.clone();
                            move |_evt: events::Input| {
                                this.lock_mut().first_name = input.value().into();
                            }
                        })
                    })
                }),
                html!("input" => web_sys::HtmlInputElement, {
                    .with_node!(input => {
                        .attr("type", "text")
                        .attr_signal("value", this.signal_cloned().map(|person| person.last_name))
                        .event({
                            let this = this.clone();
                            move |_evt: events::Input| {
                                this.lock_mut().last_name = input.value().into();
                            }
                        })
                    })
                })
            ])
        })
    }
}

fn render_people(people: &MutableVec<Person>) -> Dom {
    html!("div", {
        .children_signal_vec(people.signal_vec_cloned().map(|person| {
        }))
    })
}

/// Render the date with the `:` flashing on and off every second into `el`.
async fn use_db(el: web_sys::Element) {
    let app = App::new();
    dominator::append_dom(&dominator::get_id("app"), App::render_mutable(app));
}

use chrono::{prelude::*, Utc};
use dominator::{clone, events, html, with_node, Dom};
use futures_signals::map_ref;
use futures_signals::signal::{Mutable, Signal, SignalExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;

use crate::app::{AppInner, Route, ROUTE};
use crate::util::trim;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub title: Mutable<String>,
    pub completed: Mutable<bool>,

    #[serde(skip)]
    editing: Mutable<Option<String>>,
}

impl Todo {
    pub fn new(title: String) -> Arc<Self> {
        Arc::new(Self {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            title: Mutable::new(title),
            completed: Mutable::new(false),
            editing: Mutable::new(None),
        })
    }

    fn set_completed(&self, app: &AppInner, completed: bool) {
        self.completed.set_neq(completed);

        let todo = (*self).clone();
        let trans = app.db.transaction_readwrite(&["todos"]).unwrap_throw();
        wasm_bindgen_futures::spawn_local(async move {
            let store = trans.object_store("todos").unwrap_throw();
            store.put(&todo, None, true).await.unwrap_throw();
        });
    }

    fn remove(&self, app: &AppInner) {
        app.remove_todo(self);
    }

    fn is_visible(&self) -> impl Signal<Item = bool> {
        (map_ref! {
            let route = ROUTE.with(|r| r.signal()),
            let completed = self.completed.signal() =>
            match *route {
                Route::Active => !completed,
                Route::Completed => *completed,
                Route::All => true,
            }
        })
        .dedupe()
    }

    fn is_editing(&self) -> impl Signal<Item = bool> {
        self.editing.signal_ref(|x| x.is_some()).dedupe()
    }

    fn cancel_editing(&self) {
        self.editing.set_neq(None);
    }

    fn done_editing(&self, app: &AppInner) {
        if let Some(title) = self.editing.replace(None) {
            if let Some(title) = trim(&title) {
                self.title.set_neq(title.to_string());
            } else {
                app.remove_todo(self);
            }
        }
    }

    pub fn render(todo: Arc<Self>, app: Arc<AppInner>) -> Dom {
        html!("li", {
            .class_signal("editing", todo.is_editing())
            .class_signal("completed", todo.completed.signal())

            .visible_signal(todo.is_visible())

            .children(&mut [
                html!("div", {
                    .class("view")
                    .children(&mut [
                        html!("input" => HtmlInputElement, {
                            .class("toggle")
                            .attr("type", "checkbox")
                            .prop_signal("checked", todo.completed.signal())

                            .with_node!(element => {
                                .event(clone!(todo, app => move |_: events::Change| {
                                    todo.set_completed(&app, element.checked());
                                }))
                            })
                        }),

                        html!("label", {
                            .event(clone!(todo => move |_: events::DoubleClick| {
                                todo.editing.set_neq(Some(todo.title.get_cloned()));
                            }))

                            .text_signal(todo.title.signal_cloned())
                        }),

                        html!("button", {
                            .class("destroy")
                            .event(clone!(todo, app => move |_: events::Click| {
                                todo.remove(&app);
                            }))
                        }),
                    ])
                }),

                html!("input" => HtmlInputElement, {
                    .class("edit")

                    .prop_signal("value", todo.editing.signal_cloned()
                        .map(|x| x.unwrap_or_else(|| "".to_owned())))

                    .visible_signal(todo.is_editing())
                    .focused_signal(todo.is_editing())

                    .with_node!(element => {
                        .event(clone!(todo => move |event: events::KeyDown| {
                            match event.key().as_str() {
                                "Enter" => {
                                    element.blur().unwrap_throw();
                                },
                                "Escape" => {
                                    todo.cancel_editing();
                                },
                                _ => {}
                            }
                        }))
                    })

                    .with_node!(element => {
                        .event(clone!(todo => move |_: events::Input| {
                            todo.editing.set_neq(Some(element.value()));
                        }))
                    })

                    .event(clone!(todo, app => move |_: events::Blur| {
                        todo.done_editing(&app);
                    }))
                }),
            ])
        })
    }
}

impl PartialEq<Todo> for Todo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

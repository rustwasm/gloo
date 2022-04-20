use dominator::{
    clone, events, html, link, routing, text, text_signal, with_node, Dom, EventOptions,
};
use futures::StreamExt;
use futures_signals::signal::{Broadcaster, Mutable, Signal, SignalExt};
use futures_signals::signal_vec::{MutableVec, SignalVec, SignalVecExt};
use gloo::storage::indexed_db as idb;
use std::{collections::HashSet, pin::Pin, sync::Arc};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlInputElement, Url};

use crate::todo::Todo;
use crate::util::trim;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Active,
    Completed,
    All,
}

impl Route {
    // This could use more advanced URL parsing, but it isn't needed
    pub fn from_url(url: &str) -> Self {
        let url = Url::new(&url).unwrap_throw();
        match url.hash().as_str() {
            "#/active" => Route::Active,
            "#/completed" => Route::Completed,
            _ => Route::All,
        }
    }

    pub fn to_url(&self) -> &'static str {
        match self {
            Route::Active => "#/active",
            Route::Completed => "#/completed",
            Route::All => "#/",
        }
    }
}

impl Default for Route {
    fn default() -> Self {
        // Create the Route based on the current URL
        Self::from_url(&routing::url().lock_ref())
    }
}

thread_local! {
    pub static ROUTE: Broadcaster<Pin<Box<dyn Signal<Item = Route>>>>
        = Broadcaster::new(routing::url().signal_ref(|s| Route::from_url(s)).boxed_local());
}

#[derive(Debug, Clone)]
pub enum App {
    Loading,
    Running(Arc<AppInner>),
    Error(Arc<String>),
}

#[derive(Debug)]
pub struct AppInner {
    pub db: idb::Database,
    new_todo_title: Mutable<String>,
    todo_list: MutableVec<Arc<Todo>>,
}

impl App {
    pub fn new() -> Arc<Mutable<Self>> {
        Arc::new(Mutable::new(Self::Loading))
    }

    pub fn init(app: &Arc<Mutable<Self>>) {
        let app = (*app).clone();
        wasm_bindgen_futures::spawn_local(async move {
            let db = handle_err!(
                app,
                idb::Database::open(
                    "gloo-indexedb-todomvc",
                    1,
                    |db| {
                        let todos_store = db
                            .create_object_store(
                                "todos",
                                idb::ObjectStoreOptions::new().key_path("id"),
                            )
                            .expect_throw("creating database");
                        todos_store
                            .create_index(
                                "todos_created_at",
                                idb::IndexOptions::new().key_path("created_at"),
                            )
                            .expect_throw("creating database");
                    },
                    true
                )
                .await
            );

            // Get existing TODOs
            let trans = db.transaction_readonly(&["todos"]).unwrap_throw();
            let store = trans.object_store("todos").unwrap_throw();
            let created_at_idx = store
                .index("todos_created_at")
                .unwrap_throw()
                .unwrap_throw();
            app.replace(App::Running(Arc::new(AppInner {
                db,
                new_todo_title: Mutable::new("".into()),
                todo_list: created_at_idx
                    .get_all(&idb::Query::ALL, None)
                    .await
                    .unwrap_throw(),
            })));
        })
    }

    pub fn render(self) -> Dom {
        match self {
            App::Loading => text("loading"),
            App::Running(inner) => inner.render(),
            App::Error(msg) => text(&msg),
        }
    }
}

impl AppInner {
    fn create_new_todo(&self) {
        let mut title = self.new_todo_title.lock_mut();

        // Only create a new Todo if the text box is not empty
        let todo = if let Some(trimmed) = trim(&title) {
            let todo = Todo::new(trimmed.to_string());
            *title = "".to_string();
            todo
        } else {
            return;
        };

        let todo_js = serde_wasm_bindgen::to_value(&todo).unwrap_throw();
        self.todo_list.lock_mut().push_cloned(todo);

        let trans = self.db.transaction_readwrite(&["todos"]).unwrap_throw();
        wasm_bindgen_futures::spawn_local(async move {
            let store = trans.object_store("todos").unwrap_throw();
            store.add_raw(&todo_js, None, true).await.unwrap_throw();
        });
    }

    pub fn remove_todo(&self, todo: &Todo) {
        self.todo_list.lock_mut().retain(|x| **x != *todo);
        let trans = self.db.transaction_readwrite(&["todos"]).unwrap_throw();
        let id = todo.id.to_string();
        wasm_bindgen_futures::spawn_local(async move {
            let store = trans.object_store("todos").unwrap_throw();
            store.delete(&*id, true).await.unwrap_throw();
        });
    }

    fn remove_all_completed_todos(&self) {
        let mut ids = HashSet::new();
        self.todo_list.lock_mut().retain(|todo| {
            if todo.completed.get() == false {
                true
            } else {
                ids.insert(todo.id);
                false
            }
        });
        let trans = self.db.transaction_readwrite(&["todos"]).unwrap_throw();
        wasm_bindgen_futures::spawn_local(async move {
            let store = trans.object_store("todos").unwrap_throw();
            for id in ids {
                let id = id.to_string();
                store.delete(&*id, true).await.unwrap_throw();
            }
        });
    }

    fn set_all_todos_completed(&self, checked: bool) {
        for todo in self.todo_list.lock_ref().iter() {
            todo.completed.set_neq(checked);
        }

        // Change `completed` of all todos in the db.
        let trans = self.db.transaction_readwrite(&["todos"]).unwrap_throw();
        wasm_bindgen_futures::spawn_local(async move {
            let store = trans.object_store("todos").unwrap_throw();
            let mut cursor = store.cursor(idb::CursorOptions::default()).unwrap_throw();
            while let Some(obj) = cursor.next().await {
                let obj = obj.unwrap_throw();
                let val: Todo = obj.value().unwrap_throw();
                val.completed.set(true);
                obj.update(&val, true).await.unwrap_throw();
            }
        });
    }

    fn completed(&self) -> impl SignalVec<Item = bool> {
        self.todo_list
            .signal_vec_cloned()
            .map_signal(|todo| todo.completed.signal())
    }

    fn completed_len(&self) -> impl Signal<Item = usize> {
        self.completed().filter(|completed| *completed).len()
    }

    fn not_completed_len(&self) -> impl Signal<Item = usize> {
        self.completed().filter(|completed| !completed).len()
    }

    fn has_todos(&self) -> impl Signal<Item = bool> {
        self.todo_list
            .signal_vec_cloned()
            .len()
            .map(|len| len > 0)
            .dedupe()
    }

    fn render_header(app: Arc<Self>) -> Dom {
        html!("header", {
            .class("header")
            .children(&mut [
                html!("h1", {
                    .text("todos")
                }),

                html!("input" => HtmlInputElement, {
                    .focused(true)
                    .class("new-todo")
                    .attr("placeholder", "What needs to be done?")
                    .prop_signal("value", app.new_todo_title.signal_cloned())

                    .with_node!(element => {
                        .event(clone!(app => move |_: events::Input| {
                            app.new_todo_title.set_neq(element.value());
                        }))
                    })

                    .event_with_options(&EventOptions::preventable(), clone!(app => move |event: events::KeyDown| {
                        if event.key() == "Enter" {
                            event.prevent_default();
                            app.create_new_todo();
                        }
                    }))
                }),
            ])
        })
    }

    fn render_main(app: Arc<Self>) -> Dom {
        html!("section", {
            .class("main")

            .visible_signal(app.has_todos())

            .children(&mut [
                html!("input" => HtmlInputElement, {
                    .class("toggle-all")
                    .attr("id", "toggle-all")
                    .attr("type", "checkbox")
                    .prop_signal("checked", app.not_completed_len().map(|len| len == 0).dedupe())

                    .with_node!(element => {
                        .event(clone!(app => move |_: events::Change| {
                            app.set_all_todos_completed(element.checked());
                        }))
                    })
                }),

                html!("label", {
                    .attr("for", "toggle-all")
                    .text("Mark all as complete")
                }),

                html!("ul", {
                    .class("todo-list")
                    .children_signal_vec(app.todo_list.signal_vec_cloned()
                        .map(clone!(app => move |todo| Todo::render(todo, app.clone()))))
                }),
            ])
        })
    }

    fn render_button(text: &str, route: Route) -> Dom {
        html!("li", {
            .children(&mut [
                link!(route.to_url(), {
                    .text(text)
                    .class_signal("selected", ROUTE.with(|r| r.signal().map(move |x| x == route)))
                })
            ])
        })
    }

    fn render_footer(app: Arc<Self>) -> Dom {
        html!("footer", {
            .class("footer")

            .visible_signal(app.has_todos())

            .children(&mut [
                html!("span", {
                    .class("todo-count")

                    .children(&mut [
                        html!("strong", {
                            .text_signal(app.not_completed_len().map(|len| len.to_string()))
                        }),

                        text_signal(app.not_completed_len().map(|len| {
                            if len == 1 {
                                " item left"
                            } else {
                                " items left"
                            }
                        })),
                    ])
                }),

                html!("ul", {
                    .class("filters")
                    .children(&mut [
                        Self::render_button("All", Route::All),
                        Self::render_button("Active", Route::Active),
                        Self::render_button("Completed", Route::Completed),
                    ])
                }),

                html!("button", {
                    .class("clear-completed")

                    // Show if there is at least one completed item.
                    .visible_signal(app.completed_len().map(|len| len > 0).dedupe())

                    .event(clone!(app => move |_: events::Click| {
                        app.remove_all_completed_todos();
                    }))

                    .text("Clear completed")
                }),
            ])
        })
    }

    pub fn render(self: Arc<Self>) -> Dom {
        html!("section", {
            .class("todoapp")

            .children(&mut [
                Self::render_header(self.clone()),
                Self::render_main(self.clone()),
                Self::render_footer(self.clone()),
            ])
        })
    }
}

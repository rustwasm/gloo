use gloo_storage::indexed_db::{delete_db, Db, DbUpgrade, Error};
use serde::Deserialize;
use wasm_bindgen::UnwrapThrowExt;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn create_db() {
    delete_db("dbname").await;
    Db::open("dbname", 1, |_| ()).await.unwrap_throw();
}

fn db_upgrade(db: DbUpgrade) {
    if db.old_version < 1 && db.new_version >= 1 {
        db.create_object_store("name")
            .auto_increment(true)
            .key_path("id")
            .build()
            .unwrap_throw();
    }
    if db.old_version < 2 && db.new_version >= 2 {
        db.delete_object_store("name").unwrap_throw();
    }
}

#[wasm_bindgen_test]
async fn create_delete_object_store() {
    delete_db("dbname").await;
    let db = Db::open("dbname", 1, db_upgrade).await.unwrap_throw();
    assert_eq!(db.object_store_names_vec(), vec!["name"]);
    drop(db);
    let db = Db::open("dbname", 2, db_upgrade).await.unwrap_throw();
    assert!(db.object_store_names().length() == 0);
}

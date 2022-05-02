use gloo_storage::indexed_db as idb;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn create_db() {
    idb::delete_database("create_db", true)
        .await
        .expect("deleting db");
    idb::Database::open("create_db", 1, |_| (), true)
        .await
        .expect("opening db");
}

#[wasm_bindgen_test]
async fn create_delete_object_store() {
    fn db_upgrade(db: idb::DatabaseDuringUpgrade) {
        if db.old_version() < 1 && db.new_version() >= 1 {
            db.create_object_store(
                "name",
                idb::ObjectStoreOptions::default()
                    .auto_increment(true)
                    .key_path("id"),
            )
            .unwrap();
        }
        if db.old_version() < 2 && db.new_version() >= 2 {
            db.delete_object_store("name").unwrap();
        }
    }

    idb::delete_database("create_delete_object_store", false)
        .await
        .unwrap();
    let db = idb::Database::open("create_delete_object_store", 1, db_upgrade, false)
        .await
        .unwrap();
    assert_eq!(
        db.object_store_names().into_iter().collect::<Vec<_>>(),
        vec!["name"]
    );
    drop(db);
    let db = idb::Database::open("create_delete_object_store", 2, db_upgrade, false)
        .await
        .unwrap();
    assert!(db.object_store_names().is_empty());
}

#[wasm_bindgen_test]
async fn get_upgrade_transaction() {
    fn db_upgrade(db: idb::DatabaseDuringUpgrade) {
        if db.old_version() < 1 && db.new_version() >= 1 {
            db.create_object_store(
                "name",
                idb::ObjectStoreOptions::default()
                    .auto_increment(true)
                    .key_path("id"),
            )
            .unwrap();
        }
        if db.old_version() < 2 && db.new_version() >= 2 {
            let store = db.transaction().object_store("name").unwrap();
            store
                .create_index("name", idb::IndexOptions::default().key_path("name"))
                .unwrap();
        }
    }
    idb::delete_database("get_upgrade_transaction", false)
        .await
        .unwrap();
    let db = idb::Database::open("get_upgrade_transaction", 1, db_upgrade, false)
        .await
        .unwrap();
    drop(db);
    let _db = idb::Database::open("get_upgrade_transaction", 2, db_upgrade, false)
        .await
        .unwrap();
}

#[wasm_bindgen_test]
async fn object_store_methods() {}

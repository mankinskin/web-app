use seed::storage;
use serde_json;
use plans::{
    user::*,
};

const STORAGE_KEY: &str = "secret";

pub fn load_session() -> Option<UserSession> {
    local_storage()
        .get_item(STORAGE_KEY)
        .expect("try to get local storage item failed")
        .map(|serialized_item| {
            serde_json::from_str(&serialized_item).expect("session deserialization failed")
        })
}

pub fn store_session(session: &UserSession) {
    storage::store_data(&local_storage(), STORAGE_KEY, session);
}

pub fn delete_app_data() {
    local_storage()
        .remove_item(STORAGE_KEY)
        .expect("remove item from local storage failed");
}

// ====== PRIVATE ======

fn local_storage() -> storage::Storage {
    storage::get_storage().expect("get local storage failed")
}

// ====== ====== TESTS ====== ======

#[cfg(test)]
pub mod tests {
    use super::*;
    use plans::{
        user::*,
    };
    use rql::{
        Id,
    };
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn clean_local_storage() {
        local_storage().clear().expect("clear storage failed");
    }

    #[wasm_bindgen_test]
    fn load_session_none_test() {
        // ====== ARRANGE ======
        clean_local_storage();

        // ====== ACT & ASSERT ======
        assert!(load_session().is_none())
    }

    #[wasm_bindgen_test]
    fn store_view_test() {
        // ====== ARRANGE ======
        clean_local_storage();

        let session = UserSession {
            user_id: Id::new(),
            token: String::new(),
        };

        // ====== ACT ======
        store_session(&session);

        //====== ASSERT ======
        assert!(load_session().is_some());
    }

    #[wasm_bindgen_test]
    fn delete_app_data_test() {
        // ====== ARRANGE ======
        clean_local_storage();

        let session = UserSession {
            user_id: Id::new(),
            token: String::new(),
        };
        store_session(&session);

        // ====== ACT ======
        delete_app_data();

        // ====== ASSERT ======
        assert!(load_session().is_none());
    }
}

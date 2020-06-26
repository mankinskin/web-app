
// ====== ====== TESTS ====== ======

#[cfg(test)]
pub mod tests {
    use super::*;
    use rql::{
        Id,
    };
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn load_session_none_test() {
        // ====== ARRANGE ======
        clean_storage();

        // ====== ACT & ASSERT ======
        assert!(load_session().is_none())
    }

    #[wasm_bindgen_test]
    fn store_view_test() {
        // ====== ARRANGE ======
        clean_storage();

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
        clean_storage();

        let session = UserSession {
            user_id: Id::new(),
            token: String::new(),
        };
        store_session(&session);

        // ====== ACT ======
        clean_storage();

        // ====== ASSERT ======
        assert!(load_session().is_none());
    }
}

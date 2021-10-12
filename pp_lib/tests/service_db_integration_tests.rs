mod db_tests {
    use env_logger::Env;
    use std::{thread, time};

    use pp_lib::factory;
    use pp_lib::model;
    use pp_lib::model::Work;
    use pp_lib::service;

    #[test]
    fn test_crud_work() {
        let env = Env::default()
            .filter_or("MY_LOG_LEVEL", "info")
            .write_style_or("MY_LOG_STYLE", "always");
        env_logger::init_from_env(env);

        // given a db client
        let mut db = factory::db_client();
        // given a work request
        let work: Work = factory::generate_random_work("testdb");

        // CREATE
        // when creating work
        let res_c = service::db::create_work(&mut db, work.clone());

        // then we get the expected work struct
        assert!(!res_c.is_err());
        let work_output = res_c.unwrap();
        assert_eq!(work_output.work_code, work.work_code);
        assert_eq!(work_output.add_up_to, work.add_up_to);
        println!("\n\n>>> CREATED: {:?}\n\n", work_output);

        // RETRIEVE
        // when we retrieve that work
        let res_r = service::db::retrieve_work(&mut db, work_output.id);

        // then we get that same work struct
        assert!(!res_r.is_err());
        let work_retrieved = res_r.unwrap();
        assert_eq!(work_output, work_retrieved);
        println!("\n\n>>> RETRIEVED: {:?}\n\n", work_retrieved);

        // SEARCH
        // when we search for that work
        let res_s = service::db::search_work(&mut db, work_output.work_code.as_str());

        // then we get that same work struct
        assert!(!res_s.is_err());
        let res_list = res_s.unwrap();
        let work_searched = res_list.get(0).unwrap();
        assert_eq!(work_output, *work_searched);
        println!("\n\n>>> SEARCHED: {:?}\n\n", work_searched);

        // UPDATE
        thread::sleep(time::Duration::from_millis(500)); // so we can compare the `updated_on` field
        let res_u = service::db::update_work_done(&mut db, work_output.id);
        assert!(res_u.is_ok());

        // then check the `updated_on` field is greater than `created_on`
        let res_r = service::db::retrieve_work(&mut db, work_output.id);
        let work_updated = res_r.unwrap();
        println!("\n\n>>> UPDATED: {:?}\n\n", work_updated);
        assert!(work_updated.done);
        assert!(work_updated.updated_on > work_updated.created_on);

        // close DB connection
        let res_db_c = db.close();
        assert!(res_db_c.is_ok());
    }

    #[test]
    fn test_crud_event() {
        // given a db client
        let mut db = factory::db_client();
        // given an event
        let event = factory::new_event("my_work_code", model::VAR_COMPUTE_START, "");

        // when flushing the event to DB
        let res_ef = service::db::create_event(&mut db, event);

        // then
        assert!(res_ef.is_ok());

        // close DB connection
        let res_db_c = db.close();
        assert!(res_db_c.is_ok());
    }
}

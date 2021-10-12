mod queue_tests {
    use env_logger::Env;

    use pp_lib::factory;
    use pp_lib::service::queue;

    #[test]
    fn test_queue() {
        let env = Env::default()
            .filter_or("MY_LOG_LEVEL", "info")
            .write_style_or("MY_LOG_STYLE", "always");
        env_logger::init_from_env(env);

        let wd = factory::generate_random_work_demand();
        let res_pub = queue::publish(&wd);
        assert!(res_pub.is_ok());

        let res_sub = queue::consume_amqp_queue(1);
        assert!(res_sub.is_ok());
        let wd_list = res_sub.unwrap();
        assert_eq!(1, wd_list.len());
        assert_eq!(wd, wd_list[0]);
    }
}

use std::{thread, time};

use env_logger::Env;
use job_scheduler::{Job, JobScheduler};
use log;
use pp_lib::service::{db, queue};
use pp_lib::{factory, model};

static WORK_CODE: &str = "consumer";

fn pull_one_messge() -> model::WorkDemand {
    let res_sub = queue::consume_amqp_queue(1);
    let wd_list = res_sub.unwrap();
    let wd = wd_list[0];
    wd
}

// TODO figure out strategies with threads and multiple messages
fn consumer_process(consumer_id: String) {
    log::info!("C-{}: COMMENCE", consumer_id);
    // pull 1 message
    //
    // TODO RETURN HERE IF THE QUEUE IS EMPTY!!!
    // to avoid leaking threads...
    //
    let wd: model::WorkDemand = pull_one_messge();
    log::info!("C-{}: Pulled work demand: {:?}", consumer_id, wd);

    // map work demand to work
    let w: model::Work = factory::map_to_work(wd, WORK_CODE);
    let wc_clone = w.work_code.clone();
    log::info!("C-{}: Mapped it to work: {:?}", consumer_id, w);

    // TODO DB connection pool: https://github.com/sfackler/r2d2-postgres
    let mut db = factory::db_client();

    // insert row in table `work`
    let res_c = db::create_work(&mut db, w.clone());

    // insert row in table `events` to signal: start working
    let e_c_start = factory::new_event(wc_clone.as_str(), model::VAR_COMPUTE_START, "");
    let res_ef = db::create_event(&mut db, e_c_start);

    // do the work demand computation
    log::info!("C-{}: Starting the calculations", consumer_id);
    let mut total_value = 0;
    for n in 1..w.add_up_to {
        // hard work here...
        total_value = total_value + n;
        thread::sleep(time::Duration::from_millis(100));
    }
    log::info!(
        "C-{}: Done with calculations, result: {:?}",
        consumer_id,
        total_value
    );

    // insert row in table `events` to signal: stop working
    let e_c_stop = factory::new_event(wc_clone.as_str(), model::VAR_COMPUTE_STOP, "");
    let res_ef = db::create_event(&mut db, e_c_stop);

    // update `work` with done=true (`updated_on` field as well...)
    let res_u = db::update_work_done(&mut db, w.id);

    // insert row in table `events` to signal: computation outcome
    let e_cr = factory::new_event(
        wc_clone.as_str(),
        model::VAR_COMPUTE_RESULT,
        format!("{}", total_value).as_str(),
    );
    let res_ef = db::create_event(&mut db, e_cr);

    // close DB connection
    let res_db_c = db.close();
    assert!(res_db_c.is_ok());
    log::info!("C-{}: DONE", consumer_id);
}

fn main() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info")
        .write_style_or("MY_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let n_seconds: u8 = 4; // within 1 minute: 60 seconds
    let task_schedule = format!("1/{} * * * * *", n_seconds);

    let mut sched = JobScheduler::new();

    sched.add(Job::new(
        task_schedule.as_str().parse().unwrap(),
        move || {
            // TODO thread builder:
            // let builder = thread::Builder::new()
            // .name("foo".into());
            // let handler = builder.spawn(|| {
            // assert_eq!(thread::current().name(), Some("foo"))
            // }).unwrap();
            // TODO alternative: thread pools + Tokio tasks
            // https://www.youtube.com/watch?v=2WXNY1ppTzY
            thread::spawn(|| {
                // TODO retrieve the thread name from the builder!
                // let t_id = thread::current().name().unwrap();
                consumer_process(factory::rand_alphanumeric_any(5));
            });
        },
    ));

    loop {
        sched.tick();

        std::thread::sleep(time::Duration::from_millis(500));
    }
}

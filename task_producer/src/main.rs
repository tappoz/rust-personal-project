use env_logger::Env;
use job_scheduler::{Job, JobScheduler};
use log;
use pp_lib::factory;
use pp_lib::service::queue;
use std::time::Duration;

fn main() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info")
        .write_style_or("MY_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let n_seconds: u8 = 4; // within 1 minute: 60 seconds
    let task_schedule = format!("1/{} * * * * *", n_seconds);

    let mut sched = JobScheduler::new();

    sched.add(Job::new(task_schedule.as_str().parse().unwrap(), || {
        let wd = factory::generate_random_work_demand();
        log::info!("Generated this work to do: {:?}", wd);
        let res_pub = queue::publish(&wd);
        if res_pub.is_ok() {
            log::info!("Publish this work to do: {:?}", wd);
        } else {
            log::error!("Did NOT publish work to do: {:?}", wd);
        }
    }));

    loop {
        sched.tick();

        std::thread::sleep(Duration::from_millis(500));
    }
}

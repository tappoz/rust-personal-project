use log;
use std::env;
use std::iter::repeat_with;

use amiquip::Connection;
use chrono::{SubsecRound, Utc};
use fastrand;
use postgres::{Client, NoTls};

use super::model;
use super::service::db;
use super::service::queue;

pub fn db_client() -> Client {
    match env::var("DOCKER_DB_HOST") {
        Ok(docker_db_host) => {
            let conn_str = db::DB_CONNECTION_STR.replace("localhost", docker_db_host.as_str());
            log::info!("Using docker network DB connection string: {}", conn_str);
            let db = Client::connect(conn_str.as_str(), NoTls).unwrap();
            return db;
        }
        Err(err) => {
            log::info!("Using default DB connection string, error: {}", err);
            let db = Client::connect(db::DB_CONNECTION_STR, NoTls).unwrap();
            return db;
        }
    }
}

pub fn rand_alphanumeric() -> String {
    let length = 10;
    return repeat_with(fastrand::alphanumeric).take(length).collect();
}

pub fn rand_alphanumeric_any(length: usize) -> String {
    return repeat_with(fastrand::alphanumeric).take(length).collect();
}

pub fn rand_num() -> i32 {
    return fastrand::i32(1..100);
}

// https://github.com/jgallagher/amiquip/blob/master/examples/work_queues_new_task.rs
// https://github.com/jgallagher/amiquip/blob/master/examples/work_queues_worker.rs
pub fn amqp_connection() -> amiquip::Connection {
    let connection = Connection::insecure_open(queue::QUEUE_CONNECTION_STR).unwrap();
    let props = connection.server_properties();
    log::info!(
        "Supplying AMQP connection for cluster {:?} version {:?}",
        props.get("cluster_name"),
        props.get("version")
    );
    connection
}

pub fn new_work(work_code: &str, add_up_to: i32) -> model::Work {
    // Postgres TIMESTAMPTZ has 6 decimals
    // chrono DateTime<Utc> has 9 decimals...
    let now = Utc::now().round_subsecs(6);
    model::Work {
        id: 0,
        work_code: String::from(work_code),
        add_up_to: add_up_to,
        done: false,
        updated_on: Some(now),
        created_on: Some(now),
    }
}

pub fn generate_random_work(work_code_prefix: &str) -> model::Work {
    let w_id: String = format!("{}-{}", work_code_prefix, rand_alphanumeric());
    let work_code: &str = w_id.as_str();
    let work_add_up_to: i32 = rand_num();
    let work: model::Work = new_work(work_code, work_add_up_to);
    work
}

pub fn generate_random_work_demand() -> model::WorkDemand {
    let work_add_up_to: i32 = rand_num();
    let wd = model::WorkDemand {
        add_up_to: work_add_up_to,
        done: false,
    };
    wd
}

/// This is for consumers e.g. AMQP consumers.
/// We take the `actor_prefix` and append a random string
/// so we have a unique `work_code` identifier.
pub fn map_to_work(wd: model::WorkDemand, actor_prefix: &str) -> model::Work {
    let work_code: String = format!("{}-{}", actor_prefix, rand_alphanumeric());
    let now = Utc::now().round_subsecs(6);
    let w = model::Work {
        id: -1,
        work_code: work_code,
        add_up_to: wd.add_up_to,
        done: wd.done,
        updated_on: Some(now),
        created_on: Some(now),
    };
    w
}

pub fn new_event(work_code: &str, variable: &str, value: &str) -> model::Event {
    // Postgres TIMESTAMPTZ has 6 decimals
    // chrono DateTime<Utc> has 9 decimals...
    let now = Utc::now().round_subsecs(6);
    model::Event {
        id: 0,
        work_code: String::from(work_code),
        variable: String::from(variable),
        value: String::from(value),
        created_on: Some(now),
    }
}

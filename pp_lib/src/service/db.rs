use chrono::{DateTime, Utc};
use log;
use postgres::Client;

use crate::model::{Error, Event, Work};

// TODO move this to config files...
pub const DB_CONNECTION_STR: &'static str =
    "postgresql://postgres:test01@localhost/personalproject";

pub fn create_work(db: &mut Client, work: Work) -> Result<Work, Error> {
    // store the input data into the DB
    let rows = db.query(
        "INSERT INTO works (work_code, add_up_to, done, updated_on, created_on) VALUES ($1, $2, $3, $4, $5) RETURNING id;",
        &[&work.work_code, &work.add_up_to, &work.done, &work.updated_on, &work.created_on],
    );

    // make sure the DB process is successful
    if rows.is_err() {
        return Err(Error {
            message: format!(
                "Not able to create some work, the error: {}",
                rows.unwrap_err()
            ),
            http_code: 500,
        });
    };
    let rows_result: std::vec::Vec<postgres::Row> = rows.unwrap();
    if rows_result.len() == 0 || rows_result.len() > 1 {
        let msg: String = format!(
            "The rows returned by the DB after the creation don't have the expected length: {}",
            rows_result.len()
        );
        return Err(Error {
            message: msg,
            http_code: 500,
        });
    };

    // find out the row `id` value for the newly inserted row
    let mut id_row: Option<i32> = None;
    for row in rows_result {
        id_row = row.get("id");
        log::info!(
            "For work_code {} and work_add_up_to {}, the ID is: {:?}",
            work.work_code,
            work.add_up_to,
            id_row.unwrap()
        );
        break;
    }

    if let Some(val_id_row) = id_row {
        // TODO retrieve from DB instead?
        // alternative define lifetime for input `&mut work`
        return Ok(Work {
            id: val_id_row,
            work_code: String::from(work.work_code.as_str()),
            add_up_to: work.add_up_to,
            done: work.done,
            updated_on: work.updated_on,
            created_on: work.created_on,
        });
    }

    let msg = format!(
        "We could not find the ID coming from the DB after INSERTing a new row or work_code {} and add_up_to {}", 
        work.work_code,
        work.add_up_to,
    );
    log::error!("{}", msg);
    Err(Error {
        message: msg,
        http_code: 500,
    })
}

pub fn retrieve_work(db: &mut Client, id: i32) -> Result<Work, Error> {
    // retrieve from DB
    let rows = db.query("SELECT * FROM works WHERE id = $1;", &[&id]);

    // make sure the DB process is successful
    if rows.is_err() {
        return Err(Error {
            message: format!(
                "Not able to retrieve some work, the error: {}",
                rows.unwrap_err()
            ),
            http_code: 500,
        });
    };
    let rows_result: std::vec::Vec<postgres::Row> = rows.unwrap();
    if rows_result.len() == 0 || rows_result.len() > 1 {
        let msg: String = format!(
            "The rows returned by the DB after the retrieval don't have the expected length: {}",
            rows_result.len()
        );
        return Err(Error {
            message: msg,
            http_code: 500,
        });
    };

    // we must be having rows_result.len() == 1
    for row in rows_result {
        let id_row: i32 = row.get("id");
        let work_code_row: &str = row.get("work_code");
        let done: bool = row.get("done");
        let work_add_up_to_row: i32 = row.get("add_up_to");
        let updated_on: DateTime<Utc> = row.get("updated_on");
        let created_on: DateTime<Utc> = row.get("created_on");
        return Ok(Work {
            id: id_row,
            work_code: work_code_row.to_string(),
            done: done,
            add_up_to: work_add_up_to_row,
            updated_on: Some(updated_on),
            created_on: Some(created_on),
        });
    }

    let msg = format!("no work retrieved with id {}", id);
    log::error!("{}", msg);
    Err(Error {
        message: msg,
        http_code: 404,
    })
}

pub fn search_work(db: &mut Client, work_search: &str) -> Result<Vec<Work>, Error> {
    log::info!("Searching for DB rows with work code like {}", work_search);
    // search in DB (TODO figure out how to inject params in the LIKE operator...)
    let sql_query = format!(
        "SELECT * FROM works WHERE work_code LIKE '{}%' AND created_on > NOW() - INTERVAL '1 hour';",
        work_search,
    );
    let rows = db.query(sql_query.as_str(), &[]);

    // make sure the DB process is successful
    if rows.is_err() {
        return Err(Error {
            message: format!(
                "Not able to search for and retrieve some work, the error: {}",
                rows.unwrap_err()
            ),
            http_code: 500,
        });
    };

    // parse the results rows
    let rows_result: std::vec::Vec<postgres::Row> = rows.unwrap();
    let mut works: Vec<Work> = Vec::new();
    for row in rows_result {
        log::info!("Processing row: {:?}", row);
        let id_row: i32 = row.get("id");
        let work_code_row: &str = row.get("work_code");
        let done: bool = row.get("done");
        let work_add_up_to_row: i32 = row.get("add_up_to");
        let updated_on: DateTime<Utc> = row.get("updated_on");
        let created_on: DateTime<Utc> = row.get("created_on");
        works.push(Work {
            id: id_row,
            work_code: work_code_row.to_string(),
            done: done,
            add_up_to: work_add_up_to_row,
            updated_on: Some(updated_on),
            created_on: Some(created_on),
        });
    }

    log::info!(
        "Returning {} works for search string '{}'",
        works.len(),
        work_search
    );
    Ok(works)
}

// update `work` with done=true (`updated_on` field as well...)
pub fn update_work_done(db: &mut Client, work_id: i32) -> Result<(), String> {
    let res_upd = db.execute(
        "UPDATE works SET done = true, updated_on = CURRENT_TIMESTAMP WHERE id = $1",
        &[&work_id],
    );
    match res_upd {
        Ok(num_rows) => {
            if num_rows > 1 {
                return Err(format!(
                    "Modified more than a row for work id {}: {}",
                    work_id, num_rows
                ));
            }
            return Ok(());
        }
        Err(err) => {
            return Err(format!("Cannot update work: {}", err));
        }
    }
}

// TODO insert row in table `events`
pub fn create_event(db: &mut Client, event: Event) -> Result<(), String> {
    let res_e = db.execute(
        "
        INSERT INTO events (work_code, variable, value, created_on) 
        VALUES ($1, $2, $3, $4);
        ",
        &[
            &event.work_code,
            &event.variable,
            &event.value,
            &event.created_on,
        ],
    );
    match res_e {
        Ok(num_rows) => {
            if num_rows > 1 {
                return Err(format!("Modified more than a row for event: {}", num_rows));
            }
            return Ok(());
        }
        Err(err) => {
            return Err(format!("Cannot create event: {}", err));
        }
    }
}

// TODO retrieve rows in table `events` by `work_code` or by `variable`

use std::io::Cursor;

use lazy_static::lazy_static;
use log;
use postgres::Client;
use regex::Regex;
use serde_json;
use tiny_http::{Request, Response, StatusCode};

use pp_lib::factory;
use pp_lib::model::Work;
use pp_lib::service;

// We want to be able to recognize URL strings and extract information out of them.
// The regex library follows the RE2 standard (Golang regex https://github.com/google/re2).
// We can write our regex and test their matches on: https://regex101.com/
lazy_static! {
    // Recognize simple URL paths:
    // /work
    // /work/
    pub static ref CREATE_WORK: Regex = Regex::new("^/work/?$").unwrap();
    // Recognize URL path parameters:
    // RE2: ^/work/((?P<id>\d+?)/?)?$
    // examples:
    // /work/123/
    // /work/123
    // => extract id=123
    pub static ref RETRIEVE_WORK: Regex = Regex::new("^/work/((?P<id>\\d+?)/?)?$").unwrap();
    // Recognize URL query parameters:
    // RE2: ^/work/search/?\?(work_code=)(?P<work_code>[a-zA-Z0-9-]+?)&?$
    // examples:
    // /work/search?work_code=123
    // /work/search/?work_code=foo
    // /work/search/?work_code=foo1bar2baz3
    // => extract work_code=xxx
    pub static ref SEARCH_WORK: Regex = Regex::new("^/work/search/?\\?(work_code=)(?P<work_code>[a-zA-Z0-9-]+?)&?$").unwrap();
}

pub fn create_work(_req: &mut Request, db: &mut Client) -> Response<Cursor<Vec<u8>>> {
    let res: Response<Cursor<Vec<u8>>>;

    // generate some random "work context"
    let work: Work = factory::generate_random_work("api");

    match service::db::create_work(db, work) {
        Ok(work) => {
            res = Response::from_string(serde_json::to_string(&work).unwrap())
                .with_status_code(StatusCode(200));
        }
        Err(err) => {
            res = Response::from_string(serde_json::to_string(&err).unwrap())
                .with_status_code(StatusCode(err.http_code));
        }
    }

    return res;
}

pub fn retrieve_work(req: &mut Request, db: &mut Client) -> Response<Cursor<Vec<u8>>> {
    let res: Response<Cursor<Vec<u8>>>;

    // regex on the HTTP path to find the row ID (or row random string?)
    let req_path: &str = req.url();
    let id: i32 = RETRIEVE_WORK
        .captures(req_path)
        .and_then(|id_cap| id_cap.name("id").map(|id| id.as_str()))
        .unwrap()
        .parse::<i32>()
        .unwrap();
    log::info!("The HTTP req provided for the retrieval the id: {}", id);

    match service::db::retrieve_work(db, id) {
        Ok(work) => {
            res = Response::from_string(serde_json::to_string(&work).unwrap())
                .with_status_code(StatusCode(200));
        }
        Err(err) => {
            res = Response::from_string(serde_json::to_string(&err).unwrap())
                .with_status_code(StatusCode(err.http_code));
        }
    }
    return res;
}

pub fn search_work(req: &mut Request, db: &mut Client) -> Response<Cursor<Vec<u8>>> {
    let res: Response<Cursor<Vec<u8>>>;

    // regex on the HTTP path to find the row work_code parameter
    let req_path: &str = req.url();
    let work_code: &str = SEARCH_WORK
        .captures(req_path)
        .and_then(|work_code_cap| {
            work_code_cap
                .name("work_code")
                .map(|work_code| work_code.as_str())
        })
        .unwrap();
    log::info!(
        "The HTTP req provided for the search the work_code: {}",
        work_code
    );

    match service::db::search_work(db, work_code) {
        Ok(work) => {
            res = Response::from_string(serde_json::to_string(&work).unwrap())
                .with_status_code(StatusCode(200));
        }
        Err(err) => {
            res = Response::from_string(serde_json::to_string(&err).unwrap())
                .with_status_code(StatusCode(err.http_code));
        }
    }
    return res;
}

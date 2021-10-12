use std::io::Cursor;

use log;
use postgres::Client;
use serde_json::json;
use tiny_http::{Method, Request, Response, StatusCode};

use super::handler;

pub fn serve_routes(req: &mut Request, db: &mut Client) -> Response<Cursor<Vec<u8>>> {
    log::info!(
        "New HTTP request. Method: {:?}, URL: {:?}, Headers: {:?}",
        req.method(),
        req.url(),
        req.headers()
    );

    let res: Response<Cursor<Vec<u8>>>;

    let req_method: &Method = req.method();
    let req_path: &str = req.url();

    // match on HTTP method + HTTP paths/params
    if req_method == &Method::Post && handler::CREATE_WORK.is_match(req_path) {
        // curl -i -X POST localhost:3000/work
        res = handler::create_work(req, db)
    } else if req_method == &Method::Get && handler::RETRIEVE_WORK.is_match(req_path) {
        // curl -i -X GET localhost:3000/work/1000
        res = handler::retrieve_work(req, db)
    } else if req_method == &Method::Get && handler::SEARCH_WORK.is_match(req_path) {
        // curl -i -X GET localhost:3000/work/search?work_code=foo
        // curl -i -X GET localhost:3000/work/search?work_code=i97zMnpYNm
        res = handler::search_work(req, db)
    } else {
        // default handler like `(_, _)` when `match (&req_method, req_path)`
        res = Response::from_string(json!({"content": "route not found"}).to_string())
            .with_status_code(StatusCode(404));
    }

    return res;
}

use std::sync::Arc;
use std::thread::spawn;

use env_logger::Env;
use log;
use tiny_http::Server;

use pp_lib::factory;

mod api;
mod handler;

const HTTP_PORT: &'static str = "3000";

fn main() {
    // https://github.com/env-logger-rs/env_logger/blob/main/examples/default.rs
    //
    // The `Env` lets us tweak what the environment
    // variables to read are and what the default
    // value is if they're missing
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info")
        .write_style_or("MY_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    // sudo netstat -tunlp | grep '3000.*LISTEN'
    let server = Arc::new(Server::http(format!("0.0.0.0:{}", &HTTP_PORT)).unwrap());
    let msg = String::from(format!("Now listening on port {}", &HTTP_PORT));
    log::info!("{}", msg);

    let mut db = factory::db_client();

    let mut handles = Vec::new();
    handles.push(spawn(move || {
        for mut rq in server.incoming_requests() {
            // return the variable ownership after borrowing...
            let res = api::serve_routes(&mut rq, &mut db);

            let result = rq.respond(res);
            match result {
                Ok(result) => log::info!("Done with result: {:?}", result),
                Err(err) => log::error!("Failed to respond to request: {}", err),
            }
        }
    }));

    for h in handles {
        h.join().unwrap();
    }
}

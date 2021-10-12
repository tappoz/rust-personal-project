use structopt::StructOpt;

use pp_lib::factory;
use pp_lib::model::{Error, Work};
use pp_lib::service;

/// CLI wrapper to interact with the remote Work API via HTTP
#[derive(StructOpt, Debug)]
struct ApiArgs {
    /// The ID for the Work to retrieve
    #[structopt(required = false, long = "id", default_value = "-1")]
    id: i32,
    /// The string to search for the Work to retrieve
    #[structopt(required = false, long = "work-code", default_value = "")]
    work_code: String,
    /// Decide either for HTTP call or PgSQL call
    #[structopt(required = false, long = "call-type", default_value = "http")]
    call_type: String,
}

const CALL_TYPE: &'static [&'static str] = &["http", "db"];

// trait to perform the call
pub trait PpReq {
    fn call(&self) -> String;
}

struct HttpCall {
    id: i32,
    work_code: String,
}

impl PpReq for HttpCall {
    fn call(&self) -> String {
        let url: String;
        match self.id > 0 {
            true => url = format!("http://localhost:3000/work/{}", self.id),
            false => {
                url = format!(
                    "http://localhost:3000/work/search?work_code={}",
                    self.work_code
                )
            }
        };
        let res: ureq::Response = ureq::get(url.as_str()).call().unwrap();
        println!("For URL {} response status: {:?}", url, res.status());
        res.into_string().unwrap()
    }
}

struct PgSqlCall {
    id: i32,
    work_code: String,
}

impl PpReq for PgSqlCall {
    fn call(&self) -> String {
        // TODO make this injectable
        let mut db = factory::db_client();
        let res: Result<Vec<Work>, Error>;
        match self.id > 0 {
            true => {
                let work_res = service::db::retrieve_work(&mut db, self.id);
                let mut works: Vec<Work> = Vec::new();
                works.push(work_res.unwrap());
                res = Ok(works);
            }
            false => res = service::db::search_work(&mut db, self.work_code.as_str()),
        };

        match res {
            Ok(work) => {
                // TODO serde "to JSON"
                return format!("Retrieved via PgSQL work: {:?}", work);
            }
            Err(err) => {
                return format!("Could not find this work, error: {:?}", err);
            }
        };
    }
}

fn returns_req(args: ApiArgs) -> Box<dyn PpReq> {
    // assume here the request `ApiArgs` is valid
    if args.call_type == "http" {
        let http_call = HttpCall {
            id: args.id,
            work_code: args.work_code,
        };
        return Box::new(http_call);
    } else if args.call_type == "db" {
        let db_call = PgSqlCall {
            id: args.id,
            work_code: args.work_code,
        };
        return Box::new(db_call);
    }

    // assume HTTP
    let http_call = HttpCall {
        id: args.id,
        work_code: args.work_code,
    };
    return Box::new(http_call);
}

fn main() {
    // validation
    let args = ApiArgs::from_args();
    println!("Validating args: {:?}", args);
    if args.id <= 0 && args.id != -1 {
        println!("This ID is not valid: {}", args.id);
        std::process::exit(-1);
    }
    if args.work_code == "" && args.id <= 0 {
        println!("both `id` and `work_code` are not set!");
        std::process::exit(-1);
    }
    if args.id > 0 && args.work_code.len() > 0 {
        println!("both `id` and `work_code` are valid, we cannot use both, choose one!");
        std::process::exit(-1);
    }
    if !CALL_TYPE.contains(&args.call_type.as_str()) {
        println!("We expect valid call types: {:?}", CALL_TYPE);
        std::process::exit(-1);
    }

    let req = returns_req(args);
    let result = req.call();
    println!("DONE! {}", result);
}

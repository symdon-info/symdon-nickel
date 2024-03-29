#[macro_use] extern crate nickel;
#[macro_use] extern crate serde_derive;
extern crate dotenv;
extern crate hyper;
extern crate serde;
extern crate serde_json;

use std::env;
use std::collections::HashMap;

use dotenv::dotenv;
use nickel::{
    HttpRouter,
    JsonBody,
    Middleware,
    MiddlewareResult,
    Mountable,
    Nickel,
    Request,
    Response,
    StaticFilesHandler,
};
use hyper::header::{AccessControlAllowOrigin, AccessControlAllowHeaders};
use nickel::status::StatusCode;

struct Logger;

impl<D> Middleware<D> for Logger {
    fn invoke<'mw, 'conn>(&self, req: &mut Request<'mw, 'conn, D>, res: Response<'mw, D>) -> MiddlewareResult<'mw, D> {
        println!("logging request from logger middleware: {:?}", req.origin.uri);
        res.next_middleware()
    }
}


#[derive(Serialize, Deserialize)]
struct Person {
    firstname: String,
    lastname:  String,
}


fn enable_cors<'mw>(_req: &mut Request, mut res: Response<'mw>) -> MiddlewareResult<'mw> {
    res.set(AccessControlAllowOrigin::Any);
    res.set(AccessControlAllowHeaders(vec![
        "Origin".into(),
        "X-Requested-With".into(),
        "Content-Type".into(),
        "Accept".into(),
    ]));
    res.next_middleware()
}



fn api_handler<'mw, 'conn>(req: &mut Request<'mw, 'conn>, res: Response<'mw>) -> MiddlewareResult<'mw> {
    let person = req.json_as::<Person>().unwrap();
    res.send(serde_json::to_value(person).map_err(|e| (StatusCode::InternalServerError, e)))
}



fn root<'mw, 'conn>(_req: &mut Request<'mw, 'conn>, res: Response<'mw>) -> MiddlewareResult<'mw> {
    let mut data = HashMap::new();
    data.insert("name", "user");
    res.render("templates/index.html", &data)
}


fn main() {
    dotenv().ok();

    let mut server = Nickel::new();
    let domain = match env::var("HTTP_DOMAIN") {
        Ok(host) => host,
        Err(_) => "0.0.0.0:6767".to_string(),
    };
    let assets_path = match env::var("ASSETS_PATH") {
        Ok(path) => path,
        Err(_) => "assets".to_string(),
    };

    // setup
    // server.utilize(logger_fn);
    server.utilize(Logger);

    // root
    server.get("/", root);

    // api
    server.utilize(enable_cors);
    server.options("/api/**", middleware!(""));
    server.post("/api/", api_handler);

    // static
    server.mount("/static/", StaticFilesHandler::new(assets_path));

    match server.listen(domain) {
        Ok(_) => (),
        Err(err) => println!("Failed: {}", err)
    }
}

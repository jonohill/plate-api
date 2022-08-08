use actix_web::{HttpServer, Responder, get, HttpResponse, App, middleware};
use figment::{providers::Env, Figment};
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct Config {
    jwt_secret: String,
    listen_port: Option<u16>,
}

#[get("/ok")]
async fn ok() -> impl Responder {
    HttpResponse::Ok().body("ok")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let figment = Figment::new().merge(Env::prefixed("PLATE_API_"));
    let config: Config = figment
        .extract()
        .map_err(|e| {
            println!("{}", e);
            println!("Set config fields by prefixing environment variables with 'PLATE_API_'");
            e
        })
        .unwrap();
    let port = config.listen_port.unwrap_or(8080);

    HttpServer::new(move || {
        App::new()
            // .app_data(web::Data::new(config.clone()))
            .wrap(middleware::Compress::default())
            .service(ok)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

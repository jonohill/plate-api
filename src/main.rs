use actix_web::http::StatusCode;
use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder, ResponseError};
use figment::{providers::Env, Figment};
use plate::PlateClient;
use serde::Deserialize;

use crate::plate::PlateError;

mod plate;

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct Config {
    jwt_secret: String,
    listen_port: Option<u16>,
    user_agent: Option<String>,
}

impl ResponseError for PlateError {
    fn status_code(&self) -> StatusCode {
        match self {
            PlateError::NotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::BAD_GATEWAY,
        }
    }
}

#[get("/ok")]
async fn ok() -> impl Responder {
    HttpResponse::Ok().body("ok")
}

#[get("/vehicle/{plate}")]
async fn get_vehicle(
    plates: web::Data<PlateClient>,
    plate: web::Path<String>,
) -> Result<impl Responder, PlateError> {
    let plate = plate.into_inner();

    let vehicle = plates.search_plate(&plate).await?;

    let response = HttpResponse::Ok()
        .append_header(("Cache-Control", "max-age=604800"))
        .json(vehicle);

    Ok(response)
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

    let plate_client = web::Data::new(PlateClient::new(config.user_agent.clone()));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(plate_client.clone())
            .wrap(middleware::Compress::default())
            .service(ok)
            .service(get_vehicle)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

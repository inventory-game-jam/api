use std::{
    fs, io,
    sync::{Arc, Mutex},
};

use actix_web::{
    dev::ServiceRequest, error, get, middleware::Logger, web::Data, App, Error, HttpResponse,
    HttpServer, Responder,
};
use actix_web_httpauth::{extractors::bearer::BearerAuth, middleware::HttpAuthentication};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Player {
    pub uuid: String,
    pub score: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Team {
    pub name: String,
    pub total_score: u16,
    pub players: Vec<Player>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiConfig {
    pub valid_tokens: Vec<String>,
    pub teams: Vec<Team>,
}

impl Into<ApiState> for ApiConfig {
    fn into(self) -> ApiState {
        ApiState {
            valid_tokens: self.valid_tokens,
            teams: Arc::new(Mutex::new(self.teams)),
        }
    }
}

#[derive(Debug, Clone)]
struct ApiState {
    pub valid_tokens: Vec<String>,
    pub teams: Arc<Mutex<Vec<Team>>>,
}

async fn validate_token(
    req: ServiceRequest,
    credentials: Option<BearerAuth>,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let Some(credentials) = credentials else {
        return Err((error::ErrorBadRequest("no bearer header"), req));
    };

    let state: &Data<ApiState> = req.app_data().unwrap();

    if state
        .valid_tokens
        .contains(&credentials.token().to_string())
    {
        return Ok(req);
    }
    return Err((error::ErrorBadRequest("invalid token"), req));
}

#[get("/")]
async fn index(data: Data<ApiState>) -> impl Responder {
    HttpResponse::Ok().json(&*data.teams)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let env = env_logger::Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::init_from_env(env);

    let config: ApiConfig =
        serde_json::from_str(fs::read_to_string("./config.json").unwrap().as_str()).unwrap();
    let state: ApiState = config.into();

    HttpServer::new(move || {
        let auth = HttpAuthentication::with_fn(validate_token);

        App::new()
            .wrap(Logger::default())
            .wrap(auth)
            .app_data(Data::new(state.clone()))
            .service(index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

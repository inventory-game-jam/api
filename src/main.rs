use std::{
    fs::{self, File},
    io::{self, Write},
    sync::{Arc, Mutex},
};

use actix_files::Files;
use actix_multipart::Multipart;
use actix_web::{
    dev::ServiceRequest,
    error, get,
    middleware::Logger,
    post, put,
    web::{block, Data, Path},
    App, Error, HttpResponse, HttpServer, Responder,
};
use actix_web_httpauth::{extractors::bearer::BearerAuth, middleware::HttpAuthentication};
use futures::TryStreamExt;
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

impl From<Data<ApiState>> for ApiConfig {
    fn from(value: Data<ApiState>) -> Self {
        let teams = {
            let lock = value.teams.lock().unwrap();
            (*lock).clone()
        };

        Self {
            valid_tokens: value.valid_tokens.clone(),
            teams,
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
    let request_url = req.uri().to_string();
    if request_url.starts_with("/packs") {
        return Ok(req);
    }

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

#[post("/team_score/{team}/{score}")]
async fn team_score(data: Data<ApiState>, path: Path<(String, u16)>) -> impl Responder {
    let (team_name, score) = path.into_inner();

    {
        let mut teams = data.teams.lock().unwrap();
        for team in &mut *teams {
            if team.name == team_name {
                team.total_score += score;
                break;
            }
        }
    }

    let config: ApiConfig = data.clone().into();
    let json = serde_json::to_string_pretty(&config).unwrap();
    fs::write("./config.json", json).unwrap();

    HttpResponse::Ok().json(&*data.teams)
}

#[post("/player_score/{uuid}/{score}")]
async fn player_score(data: Data<ApiState>, path: Path<(String, u16)>) -> impl Responder {
    let (uuid, score) = path.into_inner();

    {
        let mut teams = data.teams.lock().unwrap();
        for team in &mut *teams {
            for player in &mut *team.players {
                if player.uuid == uuid {
                    team.total_score += score;
                    player.score += score;
                    break;
                }
            }
        }
    }

    let config: ApiConfig = data.clone().into();
    let json = serde_json::to_string_pretty(&config).unwrap();
    fs::write("./config.json", json).unwrap();

    HttpResponse::Ok().json(&*data.teams)
}

#[put("/pack/{name}")]
async fn upload_pack(
    path: Path<String>,
    mut form: Multipart,
) -> Result<impl Responder, actix_web::Error> {
    let name = path.into_inner();
    println!("hello");

    while let Some(mut field) = form.try_next().await? {
        let path = format!("./packs/{}", name.clone());

        let mut file = block(|| File::create(path)).await??;

        while let Some(chunk) = field.try_next().await? {
            file = block(move || file.write_all(&chunk).map(|_| file)).await??;
        }
    }

    Ok(HttpResponse::Ok().finish())
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
            .service(team_score)
            .service(player_score)
            .service(Files::new("/packs", "./packs").show_files_listing())
            .service(upload_pack)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

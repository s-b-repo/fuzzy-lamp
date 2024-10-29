use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse, Error, middleware};
use reqwest::Client;
use std::sync::Arc;
use sqlx::{SqlitePool, Row};
use actix_web::dev::Server;
use rustls::{Certificate, PrivateKey, ServerConfig};
use std::fs::File;
use std::io::{self, BufReader};

#[derive(Clone)]
struct AppState {
    client: Arc<Client>,
    db_pool: SqlitePool,
    session_required: bool, // Whether session token is required
}

async fn is_ip_allowed(db_pool: &SqlitePool, ip: &str) -> Result<bool, sqlx::Error> {
    let query = "SELECT EXISTS(SELECT 1 FROM allowlist WHERE ip = ?)";
    let result: (i64,) = sqlx::query_as(query)
        .bind(ip)
        .fetch_one(db_pool)
        .await?;
    
    Ok(result.0 == 1)
}

async fn forward_request(
    req: HttpRequest,
    body: web::Bytes,
    app_state: web::Data<AppState>
) -> Result<HttpResponse, Error> {
    // Get client IP
    let client_ip = req.peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Check IP allowlist
    let allowed = is_ip_allowed(&app_state.db_pool, &client_ip).await.unwrap_or(false);
    if !allowed {
        eprintln!("Unauthorized access attempt from IP: {}", client_ip);
        return Ok(HttpResponse::Forbidden().body("Forbidden"));
    }

    // Check session token if required
    if app_state.session_required {
        let session_token = req.headers().get("Session-Token");
        if session_token.is_none() || session_token.unwrap() != "your_expected_token" {
            eprintln!("Unauthorized access due to invalid session token from IP: {}", client_ip);
            return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
        }
    }

    // Forward the request
    let forward_url = format!("https://your_target_server.com{}", req.uri());
    let mut request_builder = app_state.client
        .request(req.method().clone(), &forward_url)
        .headers(req.headers().clone());

    // Forward the body if present
    let request = if !body.is_empty() {
        request_builder.body(body.to_vec())
    } else {
        request_builder
    };

    // Send the request and await the response
    match request.send().await {
        Ok(mut response) => {
            let mut client_resp = HttpResponse::build(response.status());
            for (key, value) in response.headers().iter() {
                client_resp.insert_header((key.clone(), value.clone()));
            }
            let body = response.bytes().await?;
            Ok(client_resp.body(body))
        }
        Err(e) => {
            eprintln!("Request failed: {}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

fn load_rustls_config() -> io::Result<ServerConfig> {
    let cert_file = &mut BufReader::new(File::open("cert.pem")?);
    let key_file = &mut BufReader::new(File::open("key.pem")?);
    let cert_chain = rustls_pemfile::certs(cert_file)
        .map(|mut certs| certs.drain(..).map(Certificate).collect())?;
    let mut keys = rustls_pemfile::pkcs8_private_keys(key_file)?;
    
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, PrivateKey(keys.remove(0)))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid certificate/key"))?;
    
    Ok(config)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Set up the database connection pool
    let db_pool = SqlitePool::connect("sqlite:allowlist.db").await.unwrap();

    // Create app state with HTTP client and database pool
    let app_state = AppState {
        client: Arc::new(Client::new()),
        db_pool,
        session_required: true, // Set to false to disable session authentication
    };

    // Load TLS configuration
    let rustls_config = load_rustls_config().expect("Failed to load TLS certificates");

    // Start the HTTPS server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(middleware::Logger::default()) // Enable logging middleware
            .default_service(web::route().to(forward_request))
    })
    .bind_rustls("0.0.0.0:8443", rustls_config)?
    .run()
    .await
}

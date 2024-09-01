use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse, Error};
use reqwest::Client;
use std::sync::Arc;

async fn forward_request(req: HttpRequest, body: web::Bytes, client: web::Data<Arc<Client>>) -> Result<HttpResponse, Error> {
    // Extract the forward URL (you might want to modify this based on your routing logic)
    let forward_url = format!("https://your_target_server.com{}", req.uri());

    // Create the request builder and forward the headers and body
    let mut request_builder = client
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
            // Convert the response back to actix-web HttpResponse
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

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize the HTTP client
    let client = Arc::new(Client::new());

    // Start the server with the HTTP client in the application data
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .default_service(web::route().to(forward_request))
    })
    .workers(16) // Number of workers (adjust based on your server's resources)
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

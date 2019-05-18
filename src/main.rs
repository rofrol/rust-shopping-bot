extern crate actix_web;
extern crate env_logger;
use std::env;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use actix_web::{
    middleware, middleware::cors::Cors, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};

fn main() -> std::io::Result<()> {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    // Get the port number to listen on.
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a number");

    let host = "0.0.0.0";
    println!("Starting on {}:{}", host, port);

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(Cors::new().send_wildcard())
            .service(web::resource("/").to(index))
            .service(
                web::scope("/webhook").service(
                    web::resource("")
                        .route(web::post().to(webhook_post))
                        .route(web::get().to(webhook_get)),
                ),
            )
            .default_service(web::route().to(HttpResponse::NotFound))
    })
    .bind((host, port))?
    .run()
}

fn index(req: HttpRequest) -> &'static str {
    println!("REQ: {:?}", req);
    "Hello world!"
}

#[derive(Debug, Serialize, Deserialize)]
struct WebhookBody {
    object: String,
    entry: Vec<WebhookEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebhookEntry {
    messaging: Vec<WebhookEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebhookEvent {
    message: String,
    sender: WebhookSender,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebhookSender {
    id: String,
}

// Creates the endpoint for our webhook
fn webhook_post(body: web::Json<WebhookBody>) -> impl Responder {
    // Checks this is an event from a page subscription
    if body.object == "page" {
        // Iterates over each entry - there may be multiple if batched
        let _messages: Vec<String> = body
            .entry
            .iter()
            .filter_map(|entry| entry.messaging.first())
            .map(|webhook_event| {
                println!("{}", webhook_event.message);

                // Get the sender PSID
                let sender_psid = webhook_event.sender.id.clone();
                println!("Sender PSID: {}", sender_psid);

                webhook_event.message.clone()
            })
            .collect();

        // Returns a '200 OK' response to all requests
        HttpResponse::Ok()
            .content_type("text/html")
            .body("EVENT_RECEIVED")
    } else {
        // Returns a '404 Not Found' if event is not from a page subscription
        HttpResponse::NotFound().finish()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Hub {
    #[serde(rename = "hub.mode")]
    mode: Option<String>,
    #[serde(rename = "hub.verify_token")]
    token: Option<String>,
    #[serde(rename = "hub.challenge")]
    challenge: String,
}

// Adds support for GET requests to our webhook
fn webhook_get(hub: web::Query<Hub>) -> impl Responder {
    // Your verify token. Should be a random string.
    let f = std::fs::read_to_string("veirify_token").unwrap();
    let verify_token = f.trim();

    // Checks if a token and mode is in the query string of the request
    match (&hub.mode, &hub.token) {
        (Some(mode), Some(token)) => {
            // Checks the mode and token sent is correct
            if mode == "subscribe" && token == verify_token {
                // Responds with the challenge token from the request
                println!("WEBHOOK_VERIFIED");
                HttpResponse::Ok()
                    .content_type("text/html")
                    .body(hub.challenge.clone())
            } else {
                // Responds with '403 Forbidden' if verify tokens do not match
                HttpResponse::Forbidden().finish()
            }
        }
        // Responds with '403 Forbidden' if verify tokens do not match
        _ => HttpResponse::Forbidden().finish(),
    }
}

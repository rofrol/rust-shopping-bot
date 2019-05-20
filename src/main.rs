extern crate actix_web;
extern crate env_logger;
use std::env;

extern crate serde;
use serde::{Deserialize, Serialize};
extern crate serde_json;

extern crate bytes;
extern crate futures;
use futures::{Future, Stream};

use actix_web::{
    middleware, middleware::cors::Cors, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};

fn main() -> std::io::Result<()> {
    // DEBUG=true cargo run
    // heroku config:set DEBUG=true
    // heroku config:unset DEBUG
    let debug = match env::var("DEBUG") {
        Ok(_) => true,
        Err(_) => false,
    };
    println!("debug: {}", debug);

    env::set_var(
        "RUST_LOG",
        if debug {
            "actix_web=debug"
        } else {
            "actix_web=info"
        },
    );
    //env::set_var("RUST_BACKTRACE", "1");

    env_logger::init();

    // Get the port number to listen on.
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a number");

    let host = "0.0.0.0";
    println!("Starting on {}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(Cors::new().send_wildcard())
            .service(web::resource("/").to(index))
            .service(
                web::scope("/webhook").service(
                    web::resource("")
                        .route(if debug {
                            web::post().to_async(webhook_post_debug)
                        } else {
                            web::post().to(webhook_post)
                        })
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
    message: WebhookMessage,
    sender: WebhookSender,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebhookMessage {
    mid: String,
    seq: i32,
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebhookSender {
    id: String,
}

// curl -H "Content-Type: application/json" -X POST "http://localhost:8080/webhook" -d @message.json
// curl -H "Content-Type: application/json" -X POST "https://rust-shopping-bot.herokuapp.com/webhook" -d @message.json
// Creates the endpoint for our webhook
fn webhook_post(body: web::Json<WebhookBody>) -> impl Responder {
    webhook_post_response(body.into_inner())
}

fn webhook_post_response(body: WebhookBody) -> HttpResponse {
    // Checks this is an event from a page subscription
    if body.object == "page" {
        // Iterates over each entry - there may be multiple if batched
        let _messages: Vec<String> = body
            .entry
            .iter()
            .filter_map(|entry| entry.messaging.first())
            .map(|webhook_event| {
                // Get the sender PSID
                let sender_psid = webhook_event.sender.id.clone();
                println!("Sender PSID: {}", sender_psid);

                println!("{}", webhook_event.message.text);
                webhook_event.message.text.clone()
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

fn webhook_post_debug(
    req: HttpRequest,
    pl: web::Payload,
) -> impl Future<Item = HttpResponse, Error = actix_web::error::Error> {
    println!("REQ: {:?}", req);
    pl.concat2().from_err().and_then(|payload_body| {
        // body is loaded, now we can deserialize json-rust
        let str = std::str::from_utf8(&payload_body).unwrap();
        println!("str: {}", str);
        let body = serde_json::from_str::<WebhookBody>(&str).unwrap();
        webhook_post_response(body)
    })
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

use actix_web::{rt, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_ws::{AggregatedMessage, Session};
use futures_util::StreamExt as _;
use std::{net::SocketAddr, sync::Mutex, collections::HashMap};

const DB: &'static str = "./data/score.txt";

async fn echo(req: HttpRequest, stream: web::Payload, data: web::Data<Counter>) -> Result<HttpResponse, Error> {
    let (res, session, stream) = actix_ws::handle(&req, stream)?;

    let mut locked_sessions = data.sessions.lock().unwrap();
    locked_sessions.insert(req.peer_addr().unwrap(), session);
    drop(locked_sessions);
    
    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));
    
    rt::spawn(async move {
        while let Some(message) = stream.next().await {
            match message {
                Ok(AggregatedMessage::Text(_)) => {
                    let mut locked_counter = data.counter.lock().unwrap();
                    *locked_counter += 1;
                    let counter = *locked_counter;
                    drop(locked_counter);
                    let _ = std::fs::write(DB, format!("{counter}")); // ideally this should be async

                    let mut locked_sessions = data.sessions.lock().unwrap(); 
                    let keys: Vec<_> = locked_sessions.keys().map(|element| element.clone()).collect();
                    for address in &keys {
                        let session = locked_sessions.get_mut(&address).unwrap();
                        if session.text(format!("{counter}")).await.is_err() {
                            locked_sessions.remove(&address);
                        };
                    }
                    println!("addresses={keys:?}");
                }
                Ok(AggregatedMessage::Close(_)) => {
                    let mut locked_sessions = data.sessions.lock().unwrap(); 
                    locked_sessions.remove(&req.peer_addr().unwrap());
                }
                _ => {}
            }
        }
    });

    Ok(res)
}

struct Counter {
    counter: Mutex<i32>,
    sessions: Mutex<HashMap<SocketAddr, Session>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let counter = web::Data::new(Counter {
        counter: Mutex::new({
            let content = std::fs::read_to_string(DB).unwrap();
            content.trim().parse().unwrap()
        }),
        sessions: Mutex::new(HashMap::new()),
    });

    HttpServer::new(move || App::new().app_data(counter.clone()).route("/echo", web::get().to(echo)))
        .bind(("127.0.0.1", 8080))?.run().await
}

extern crate simple_server;
#[macro_use]
extern crate lazy_static;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, sync_channel, SyncSender};
use std::thread;

use rust_embed::RustEmbed;
use simple_server::Server;
use ws::{listen, Message};

use crate::utils::encode_loop;

mod utils;

#[derive(RustEmbed)]
#[folder = "res/"]
struct Asset;

lazy_static! {
    pub static ref chanel : (SyncSender<Message>, Arc<Mutex<Receiver<Message>>>) = make_things();
}

fn make_things() -> (SyncSender<Message>, Arc<Mutex<Receiver<Message>>>) {
    let (data_tx, data_rx): (SyncSender<Message>, Receiver<Message>) = sync_channel(0);
    let arc_rx: Arc<Mutex<Receiver<Message>>> = Arc::new(Mutex::new(data_rx));
    (data_tx, arc_rx)
}

fn main() {
    let server = Server::new(|_, mut response| {
        Ok(response.body(Vec::from(Asset::get("index.html").unwrap()))?)
    });

    thread::spawn(move || {
        println!("Listening for http connections on port 80...");
        server.listen("0.0.0.0", "80");
    });

    thread::spawn(move || {
        println!("Listening for websocket connections on port 55555...");
        listen("0.0.0.0:4444", |out| {
            let a = &chanel.1;
            thread::Builder::new()
                .name(format!("connection_handler_{}", out.connection_id()))
                .spawn(move || {
                    loop {
                        out.send(a.lock().unwrap().recv().unwrap()).unwrap();
                    }
                })
                .expect("creating thread");
            move |msg| {
                println!("Got: {}", msg);
                Ok(())
            }
        })
            .unwrap();
    });

    encode_loop();
}

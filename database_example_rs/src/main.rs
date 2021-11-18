mod dank;

use anyhow::Result;
use clap::Parser;
use colored::*;
use crossbeam::thread;
use crossbeam_channel::{unbounded, Sender};
use log::{debug, info, LevelFilter};
use redis::{Client, Connection, Msg, PubSub};
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use std::net::TcpListener;
use tungstenite::{accept, Message};

/// Simple POC to process received messages from a redis pubsub connection.
#[derive(Parser)]
#[clap(version = "dev")]
struct Opts {
    /// Host of the redis server.
    #[clap(short, long, default_value = "127.0.0.1")]
    host: String,
    /// Channel(s) to listen on.
    #[clap(short, long, value_delimiter = ',')]
    channel: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct OutgoingMessage {
    channel: String,
    message: String,
}

fn main() -> Result<()> {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .unwrap();
    let opts: Opts = Opts::parse();
    let (s, r) = unbounded();

    thread::scope(|scope| {
        scope.spawn(move |_| {
            let mut connection = establish_connection(&opts).unwrap();
            let mut pubsub = connection.as_pubsub();
            pubsub.psubscribe(&opts.channel).unwrap();

            info!("listening on the channel(s): {:?}", opts.channel);

            loop {
                match fetch_message(&mut pubsub) {
                    Ok((msg, payload)) => process_message(&s, msg, payload),
                    Err(_) => continue,
                }
            }
        });

        let server = TcpListener::bind("127.0.0.1:9001").unwrap();
        info!("websocket server started at localhost:9001");

        for stream in server.incoming() {
            scope.spawn(|_| {
                let r2 = r.clone();
                let mut websocket = accept(stream.unwrap()).unwrap();
                let peer = websocket.get_mut().peer_addr().unwrap();
                info!("established a new connection: {}", peer);

                loop {
                    match r2.recv() {
                        Ok(msg) => {
                            let message = OutgoingMessage {
                                message: msg.get_payload().unwrap(),
                                channel: msg.get_channel_name().to_string(),
                            };

                            let serialized = serde_json::to_string(&message).unwrap();
                            if let Err(_) = websocket.write_message(Message::from(serialized)) {
                                info!("removing disconnected peer: {}", peer);
                                continue;
                            }
                        }
                        Err(_) => continue,
                    }
                }
            });
        }
    })
    .unwrap();

    return Ok(());
}

fn process_message(s: &Sender<Msg>, msg: Msg, payload: String) {
    debug!("ch '{}': {}", msg.get_channel_name(), payload);
    s.send(msg).unwrap();
}

fn fetch_message(pubsub: &mut PubSub) -> Result<(Msg, String)> {
    let msg = pubsub.get_message()?;
    let payload: String = msg.get_payload()?;

    Ok((msg, payload))
}

fn establish_connection(opts: &Opts) -> Result<Box<Connection>> {
    let client = Client::open(format!("redis://{}/", opts.host))?;

    let connection = client.get_connection()?;

    info!("connected to redis on: {}", opts.host.yellow());
    Ok(Box::new(connection))
}

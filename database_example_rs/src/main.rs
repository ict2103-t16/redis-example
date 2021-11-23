use anyhow::Result;
use bus::Bus;
use clap::Parser;
use colored::*;
use crossbeam::thread;
use log::{debug, info, LevelFilter};
use redis::{Client, Connection, Msg, PubSub};
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread as rs_threads;
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

#[derive(Serialize, Deserialize, Clone, Debug)]
struct OutgoingMessage {
    channel: String,
    message: String,
}

type MessageBus = Arc<Mutex<Bus<OutgoingMessage>>>;

fn new_message_bus(length: usize) -> MessageBus {
    return Arc::new(Mutex::new(Bus::new(length)));
}

fn main() -> Result<()> {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .unwrap();
    let opts: Opts = Opts::parse();
    let bus = new_message_bus(10);
    let broadcasting_bus = bus.clone();

    thread::scope(|scope| {
        scope.spawn(|_| {
            let server = TcpListener::bind("127.0.0.1:9001").unwrap();
            info!("websocket server started at localhost:9001");

            for stream in server.incoming() {
                let mut rxb = bus.clone().lock().unwrap().add_rx();
                rs_threads::spawn(move || {
                    let mut websocket = accept(stream.unwrap()).unwrap();
                    let peer = websocket.get_mut().peer_addr().unwrap();
                    info!("established a new connection: {}", peer);

                    loop {
                        if let Ok(msg) = rxb.recv() {
                            let serialized = serde_json::to_string(&msg).unwrap();
                            if websocket.write_message(Message::from(serialized)).is_err() {
                                info!("removing disconnected peer: {}", peer);
                                break;
                            }
                        }
                    }
                    info!("thread exited for: {}", peer);
                });
            }
        });
        scope.spawn(|_| {
            let mut connection = establish_connection(&opts).unwrap();
            let mut pubsub = connection.as_pubsub();
            pubsub.psubscribe(&opts.channel).unwrap();

            info!("listening on the channel(s): {:?}", opts.channel);

            loop {
                match fetch_message(&mut pubsub) {
                    Ok((msg, payload)) => {
                        let message = OutgoingMessage {
                            message: payload,
                            channel: msg.get_channel_name().into(),
                        };
                        debug!("ch '{}': {}", message.channel, message.message);
                        broadcasting_bus.lock().unwrap().broadcast(message);
                    }
                    Err(_) => continue,
                }
            }
        });
    })
    .unwrap();

    Ok(())
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

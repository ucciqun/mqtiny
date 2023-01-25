use clap::Parser;
use futures::SinkExt;
use mqtiny::*;
use std::{collections::HashMap, error::Error, net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;

#[derive(Parser, Debug)]
struct Args {
    /// MQTiny service port
    #[arg(short, long, default_value_t = 1883)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let listener = TcpListener::bind(format!("0.0.0.0:{}", args.port)).await?;
    println!("Listening port: {}...", args.port);

    let (tx, mut rx) = mpsc::unbounded_channel();
    let clients = Arc::new(Mutex::new(HashMap::<SocketAddr, Tx>::new()));

    {
        let clients = clients.clone();
        tokio::spawn(async move {
            manage(&mut rx, clients).await;
        });
    }

    loop {
        let tx = tx.clone();
        let (stream, _) = listener.accept().await?;
        let clients = Arc::clone(&clients);

        tokio::spawn(async move {
            process(stream, tx, clients).await.unwrap();
        });
    }
}

async fn manage(rx: &mut Rx, clients: Arc<Mutex<HashMap<SocketAddr, Tx>>>) {
    let mut subscription_table = HashMap::<u16, Vec<SocketAddr>>::new();
    while let Some(cmd) = rx.recv().await {
        // match cmd {
        //     MqttPacket::Publish(_) => todo!(),
        //     MqttPacket::Subscribe(subscribe) => subscription_table
        //         .get(&subscribe.topic_name)
        //         .unwrap()
        //         .push(value),
        // }
        match cmd {
            Command::Publish { packet } => {
                if let Some(subscriptions) = subscription_table.get(&packet.topic_name) {
                    let mut clients = clients.lock().await;
                    for subscriber in subscriptions {
                        if let Some(subscriber) = clients.get_mut(subscriber) {
                            subscriber
                                .send(Command::Publish {
                                    packet: packet.clone(),
                                })
                                .unwrap();
                        }
                    }
                }
            }
            Command::Subscribe { packet, client } => {
                match subscription_table.get_mut(&packet.topic_name) {
                    Some(subscriptions) => subscriptions.push(client),
                    None => {
                        subscription_table.insert(packet.topic_name, vec![client]);
                    }
                    // Some(subscriptions) => subscriptions.push(client),
                    // None => subscription_table.insert(packet.topic_name, vec![client]),
                }
            }
        }
    }
}

async fn process(
    stream: TcpStream,
    tx_to_manager: Tx,
    clients: Arc<Mutex<HashMap<SocketAddr, Tx>>>,
) -> Result<(), Box<dyn Error>> {
    let mut framed = Framed::new(stream, MQTinyCodec {});
    let (tx, mut rx) = mpsc::unbounded_channel();
    {
        clients
            .lock()
            .await
            .insert(framed.get_ref().peer_addr()?, tx);
    }

    loop {
        tokio::select! {
            Some(msg) = rx.recv() => {
                match msg{
                    Command::Publish { packet } => {
                        framed.send(mqtiny::MqttPacket::Publish(packet)).await.unwrap();
                    },
                    _ => {},
                }
            }
            result = framed.next() => match result {
                Some(Ok(packet)) => match packet {
                    MqttPacket::Publish(publish) => {
                        tx_to_manager
                            .send(Command::Publish { packet: publish })
                            .unwrap();
                    }
                    MqttPacket::Subscribe(subscribe) => {
                        // println!("{:?}", subscribe.topic_name);
                        tx_to_manager
                            .send(Command::Subscribe {
                                packet: subscribe,
                                client: framed.get_ref().peer_addr()?,
                            })
                            .unwrap();
                    }
                    _=>{},
                },
                Some(Err(e)) => eprintln!("{}", e),
                None => break,
        }
        }
    }

    {
        let mut clients = clients.lock().await;
        clients.remove(&framed.get_ref().peer_addr()?);
        println!("this client is disconnected.");
    }
    Ok(())
}

#[derive(Debug, Clone)]
enum Command {
    Subscribe {
        packet: MqttSubscribePacket,
        client: SocketAddr,
    },
    Publish {
        packet: MqttPublishPacket,
    },
}

type Tx = mpsc::UnboundedSender<Command>;
type Rx = mpsc::UnboundedReceiver<Command>;

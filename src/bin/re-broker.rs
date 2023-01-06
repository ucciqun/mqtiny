use clap::Parser;
use futures::SinkExt;
use mqtiny::*;
use std::{collections::HashMap, error::Error, io, net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Encoder, Framed};

use bytes::{Buf, BufMut, BytesMut};
use tokio::{io::AsyncWrite, io::AsyncWriteExt};

#[derive(Parser, Debug)]
struct Args {
    /// MQTiny service port
    #[arg(short, long, default_value_t = 1883)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create the shared subscriber table.
    let subscription_table = Arc::new(Mutex::new(SubscriptionTable::new()));
    let clients = Arc::new(Mutex::new(Clients::new()));
    let args = Args::parse();

    let listener = TcpListener::bind(&format!("127.0.0.1:{}", args.port)).await?;

    println!("Listening {}...", args.port);

    loop {
        let (stream, addr) = listener.accept().await?;

        let subscription_table = Arc::clone(&subscription_table);
        let clients = Arc::clone(&clients);

        tokio::spawn(async move {
            process(subscription_table, clients, stream, addr)
                .await
                .unwrap();
        });
    }

    Ok(())
}

async fn process(
    subscription_table: Arc<Mutex<SubscriptionTable>>,
    clients: Arc<Mutex<Clients>>,
    stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let mut framed = Framed::new(stream, MQTinyCodec {});
    let mut client = Client::new(clients, framed).await?;

    loop {
        tokio::select! {
            Some(msg)=client.rx.recv()=>{
                println!("{:?}",msg);
            }
            result=client.framed.next()=>match result{
                Some(Ok(msg)) => {
                    match msg{
                        MqttPacket::Publish(publish)=>{
                            println!("{:?}",publish);
                            let subscription_table=subscription_table.lock().await;
                            let subscriptions=subscription_table.get_subscriptions(&publish.topic_name);
                            let clients=clients.lock().await;
                            for subscription in subscriptions{
                                let Some(tx)=clients.clients.get(&subscription);
                                tx.send(publish.payload.into());
                            }
                        }
                        MqttPacket::Subscribe(subscribe) => {
                            let mut subscription_table=subscription_table.lock().await;
                            subscription_table.add_subscription(subscribe.topic_name, client.framed.get_ref().peer_addr()?);
                        },
                    }
                },
                Some(Err(_))=>{},
                None => break,
            }
        }
    }

    {
        println!("{}:{} is disconnected.", addr.ip().to_string(), addr.port());
    }

    Ok(())
}

struct SubscriptionTable {
    subscriptions: HashMap<Topic, Vec<SocketAddr>>,
}
impl SubscriptionTable {
    fn new() -> SubscriptionTable {
        SubscriptionTable {
            subscriptions: HashMap::new(),
        }
    }
    fn add_subscription(&mut self, topic: Topic, client_addr: SocketAddr) {
        self.subscriptions
            .entry(topic)
            .or_insert(Vec::new())
            .push(client_addr);
    }
    fn remove_subscription(&mut self, topic: &Topic, client_addr: &SocketAddr) {
        if let Some(clients) = self.subscriptions.get_mut(topic) {
            clients.retain(|c| c != client_addr);
        }
    }
    fn get_subscriptions(&self, topic: &Topic) -> Vec<SocketAddr> {
        self.subscriptions
            .get(topic)
            .map(|clients| clients.clone())
            .unwrap_or_default()
    }
}
struct Clients {
    clients: HashMap<SocketAddr, Tx>,
}
impl Clients {
    fn new() -> Self {
        Clients {
            clients: HashMap::new(),
        }
    }
}
struct Client {
    framed: Framed<TcpStream, MQTinyCodec>,
    rx: Rx,
}
impl Client {
    async fn new(
        clients: Arc<Mutex<Clients>>,
        framed: Framed<TcpStream, MQTinyCodec>,
    ) -> io::Result<Client> {
        let addr = framed.get_ref().peer_addr()?;
        let (tx, rx) = mpsc::unbounded_channel();
        clients.lock().await.clients.insert(addr, tx);

        Ok(Client { framed, rx })
    }
}

type Tx = mpsc::UnboundedSender<Vec<u8>>;

type Rx = mpsc::UnboundedReceiver<Vec<u8>>;

type Topic = u16;

// impl Encoder<Packet> for MQTinyCodec {
//     type Error = std::io::Error;
//     fn encode(&mut self, item: Packet, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
//         let fixed_header_len = 2;
//         dst.reserve(fixed_header_len + item.fixed_header.remaining_length as usize);
//         dst.put_u8(
//             ((item.fixed_header.packet_type as u8) << 4) + ((item.fixed_header.qos as u8) << 1),
//         );
//         dst.put_u8(item.fixed_header.remaining_length);

//         match item.fixed_header.packet_type {
//             PacketType::Publish => {
//                 dst.put_u16(item.variable_header.topic_name);
//                 if let Some(payload) = item.payload {
//                     dst.put(payload);
//                 }
//             }
//             PacketType::Subscribe => {
//                 dst.put_u16(item.variable_header.topic_name);
//             }
//             PacketType::Puback => {}
//             _ => {}
//         }

//         Ok(())
//     }
// }

// fn encode(item: &Packet, dst: &mut bytes::BytesMut) -> Result<(), std::io::ErrorKind> {
//     let fixed_header_len = 2;
//     dst.reserve(fixed_header_len + item.fixed_header.remaining_length as usize);
//     dst.put_u8(((item.fixed_header.packet_type as u8) << 4) + ((item.fixed_header.qos as u8) << 1));
//     dst.put_u8(item.fixed_header.remaining_length);

//     match item.fixed_header.packet_type {
//         PacketType::Publish => {
//             dst.put_u16(item.variable_header.topic_name);
//             if let Some(payload) = item.payload {
//                 dst.put(payload);
//             }
//         }
//         PacketType::Subscribe => {
//             dst.put_u16(item.variable_header.topic_name);
//         }
//         PacketType::Puback => {}
//         _ => {}
//     }

//     Ok(())
// }

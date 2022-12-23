use std::{collections::HashMap, sync::Arc};

use bytes::{Buf, BufMut, BytesMut};
use clap::Parser;
use futures::sink::SinkExt;
use mqtiny::*;
use tokio::{
    io::AsyncWrite,
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    pin,
    sync::Mutex,
};
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Encoder, Framed, FramedRead};

const MAX_TOPIC: usize = 100;
const NEW_CONNECTION: u64 = 100;

#[derive(Parser, Debug)]
struct Args {
    /// MQTiny service port
    #[arg(short, long, default_value_t = 1883)]
    port: u16,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    let listener = TcpListener::bind(&format!("127.0.0.1:{}", args.port))
        .await
        .unwrap();
    println!("Listening on 127.0.0.1:{}", args.port);

    let table = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (client, _) = listener.accept().await.unwrap();

        let table = table.clone();

        tokio::spawn(async move {
            process(client, table).await;
        });
    }
}

async fn process(
    client: TcpStream,
    table: Arc<Mutex<HashMap<u16, Framed<TcpStream, MQTinyCodec>>>>,
) {
    let codec = MQTinyCodec {};
    let mut framed = Framed::new(client, codec);

    while let Some(frame) = framed.next().await {
        match frame {
            Ok(data) => match data.fixed_header.packet_type {
                PacketType::Publish => {
                    let mut table = table.lock().await;

                    match data.fixed_header.qos {
                        QoS::AtLeastOnce | QoS::ExactlyOnce => {
                            let data = Packet {
                                fixed_header: FixedHeader {
                                    packet_type: PacketType::Puback,
                                    qos: data.fixed_header.qos,
                                    remaining_length: 2,
                                },
                                variable_header: mqtiny::VariableHeader { topic_name: 0 },
                                payload: None,
                            };
                            framed.send(data);
                            framed.flush();
                        }
                        QoS::AtMostOnce => {}
                    }

                    if let Some(framed) = table.get_mut(&data.variable_header.topic_name) {
                        framed.send(data);
                        framed.flush();
                    }
                }
                PacketType::Subscribe => {
                    let mut table = table.lock().await;
                    table.insert(data.variable_header.topic_name, framed);
                    break;
                }
                _ => {}
            },
            Err(err) => eprintln!("error: {:?}", err),
        }
    }
}

struct MQTinyCodec {}
impl Decoder for MQTinyCodec {
    type Item = Packet;

    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 2 {
            return Ok(None);
        }

        let fixed_header = FixedHeader {
            packet_type: PacketType::from_usize(src[0] as usize >> 4).unwrap(),
            qos: QoS::from_usize((src[0] as usize >> 1) & 0b11).unwrap(),
            remaining_length: src[1],
        };
        src.advance(2);

        if src.len() < fixed_header.remaining_length as usize {
            src.reserve(fixed_header.remaining_length as usize - src.len());

            return Ok(None);
        }

        match fixed_header.packet_type {
            PacketType::Publish => {
                let variable_header = VariableHeader {
                    topic_name: src.get_u16(),
                };
                let payload = src.copy_to_bytes(fixed_header.remaining_length as usize - 2);
                Ok(Some(Packet {
                    fixed_header,
                    variable_header,
                    payload: Some(payload),
                }))
            }
            PacketType::Subscribe => {
                let variable_header = VariableHeader {
                    topic_name: src.get_u16(),
                };
                Ok(Some(Packet {
                    fixed_header,
                    variable_header,
                    payload: None,
                }))
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "undefined packet",
            )),
        }
    }
}

impl Encoder<Packet> for MQTinyCodec {
    type Error = std::io::Error;
    fn encode(&mut self, item: Packet, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        let fixed_header_len = 2;
        dst.reserve(fixed_header_len + item.fixed_header.remaining_length as usize);
        dst.put_u8(
            ((item.fixed_header.packet_type as u8) << 4) + ((item.fixed_header.qos as u8) << 1),
        );
        dst.put_u8(item.fixed_header.remaining_length);

        match item.fixed_header.packet_type {
            PacketType::Publish => {
                dst.put_u16(item.variable_header.topic_name);
                if let Some(payload) = item.payload {
                    dst.put(payload);
                }
            }
            PacketType::Subscribe => {
                dst.put_u16(item.variable_header.topic_name);
            }
            PacketType::Puback => {}
            _ => {}
        }

        Ok(())
    }
}

fn encode(item: &Packet, dst: &mut bytes::BytesMut) -> Result<(), std::io::ErrorKind> {
    let fixed_header_len = 2;
    dst.reserve(fixed_header_len + item.fixed_header.remaining_length as usize);
    dst.put_u8(((item.fixed_header.packet_type as u8) << 4) + ((item.fixed_header.qos as u8) << 1));
    dst.put_u8(item.fixed_header.remaining_length);

    match item.fixed_header.packet_type {
        PacketType::Publish => {
            dst.put_u16(item.variable_header.topic_name);
            if let Some(payload) = item.payload {
                dst.put(payload);
            }
        }
        PacketType::Subscribe => {
            dst.put_u16(item.variable_header.topic_name);
        }
        PacketType::Puback => {}
        _ => {}
    }

    Ok(())
}

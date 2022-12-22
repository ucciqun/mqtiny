use std::{collections::HashMap, sync::Arc};

use bytes::{Buf, BufMut};
use clap::Parser;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Encoder, Framed, FramedRead};

const MAX_TOPIC: usize = 100;
const NEW_CONNECTION: u64 = 100;

#[allow(unused)]
#[derive(Debug)]
enum PacketType {
    Unknown = 0,
    Connect = 1,
    Connack = 2,
    Publish = 3,
    Puback = 4,
    Pubrec = 5,
    Pubrel = 6,
    Pubcomp = 7,
    Subscribe = 8,
    Suback = 9,
    Unsubscribe = 10,
    Unsuback = 11,
    Pingreq = 12,
    Pingresp = 13,
    Disconnect = 14,
    Reserved = 15,
}
#[allow(unused)]
enum QoS {
    AtMostOnce = 0,
    AtLeastOnce = 1,
    ExactlyOnce = 2,
}

#[derive(Parser, Debug)]
struct Args {
    /// MQTiny service port
    #[arg(short, long, default_value_t = 1883)]
    port: u16,
}

#[allow(unused)]
struct Packet {
    fixed_header: FixedHeader,
    variable_header: VariableHeader,
    payload: String,
    src: Vec<u8>,
}

#[allow(unused)]
struct FixedHeader {
    packet_type: PacketType,
    qos: QoS,
    remaining_length: u8,
}

#[allow(unused)]
struct VariableHeader {
    topic_name: u16,
}
struct MQTinyCodec {}
impl Decoder for MQTinyCodec {
    type Item = Packet;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 2 {
            return Ok(None);
        }

        let packet_type = src[0] >> 4;
        let qos: QoS = match src[0] & 0b10 {
            0 => QoS::AtMostOnce,
            1 => QoS::AtMostOnce,
            2 => QoS::ExactlyOnce,
            _ => QoS::AtMostOnce,
        };
        let remaining_length = src[1] as usize;

        if src.len() < (2 + remaining_length) {
            src.reserve(2 + remaining_length - src.len());

            return Ok(None);
        }

        let mut topic_name_bytes = [0u8; 2];
        topic_name_bytes.copy_from_slice(&src[2..4]);
        let topic_name = u16::from_be_bytes(topic_name_bytes);
        let payload = src[(2 + 2)..(2 + remaining_length)].to_vec();

        // switch packet_type
        let packet_type: PacketType = match packet_type {
            0 => PacketType::Unknown,
            1 => PacketType::Connect,
            2 => PacketType::Connack,
            3 => PacketType::Publish,
            4 => PacketType::Puback,
            5 => PacketType::Pubrec,
            6 => PacketType::Pubrel,
            7 => PacketType::Pubcomp,
            8 => PacketType::Subscribe,
            9 => PacketType::Suback,
            10 => PacketType::Unsubscribe,
            11 => PacketType::Unsuback,
            12 => PacketType::Pingreq,
            13 => PacketType::Pingresp,
            14 => PacketType::Reserved,
            _ => PacketType::Unknown,
        };

        match packet_type {
            PacketType::Publish => {
                println!("publish is coming!");
                let p = Packet {
                    fixed_header: FixedHeader {
                        packet_type,
                        qos,
                        remaining_length: remaining_length.try_into().unwrap(),
                    },
                    variable_header: VariableHeader { topic_name },
                    payload: String::from_utf8(payload).unwrap(),
                    src: (src[..]).to_vec(),
                };
                src.advance(2 + remaining_length);
                Ok(Some(p))
            }
            PacketType::Subscribe => {
                println!("subscribe is coming!");
                let p = Packet {
                    fixed_header: FixedHeader {
                        packet_type,
                        qos,
                        remaining_length: remaining_length.try_into().unwrap(),
                    },
                    variable_header: VariableHeader { topic_name },
                    payload: "".to_string(),
                    src: (src[..]).to_vec(),
                };
                src.advance(2 + remaining_length);
                Ok(Some(p))
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "undefined packet type",
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
        dst.put_u8(item.fixed_header.remaining_length.into());

        match item.fixed_header.packet_type {
            PacketType::Publish | PacketType::Subscribe => {
                dst.put_u16(item.variable_header.topic_name);
                dst.extend_from_slice(item.payload.as_bytes());
            }
            PacketType::Puback => {}
            _ => {}
        }

        Ok(())
    }
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

async fn process(client: TcpStream, table: Arc<Mutex<HashMap<u16, TcpStream>>>) {
    let codec = MQTinyCodec {};
    let mut framed = Framed::new(client, codec);

    while let Some(frame) = .next().await {
        match frame {
            Ok(data) => match data.fixed_header.packet_type {
                PacketType::Publish => {
                    let mut table = table.lock().await;

                    match table.get_mut(&data.variable_header.topic_name) {
                        Some(stream) => {
                            stream.write_all(&data.src).await.unwrap();
                        }
                        _ => {}
                    }

                    match data.fixed_header.qos {
                        QoS::AtLeastOnce | QoS::ExactlyOnce => {
                            let mut response = Vec::new();
                            response.push((PacketType::Puback as u8) << 4 + (1 << 1));
                            response.push(1 as u8);
                            client.write_all(&response).await.unwrap();
                            // client.write_all(&request).await.unwrap();
                        }
                        QoS::AtMostOnce => {}
                    }
                }
                _ => {}
            },
            Err(err) => eprintln!("error: {:?}", err),
        }
    }
}

// fn main(){
//     let args=Args::parse();
//     let listener=TcpListener::bind(&format!("127.0.0.1:{}",args.port)).unwrap();
//     listener.set_nonblocking(true).unwrap();
//     let epoll_fd=epoll_create().unwrap();

//     let event=EpollEvent::new(EpollFlags::EPOLLIN|EpollFlags::EPOLLRDHUP,NEW_CONNECTION);
//     let fd=listener.as_raw_fd();
//     epoll_ctl(epoll_fd, EpollOp::EpollCtlAdd, fd, &mut event);

//     let mut events=vec![EpollEvent::empty();1024];

//     let mut clients=HashMap::new();
//     let mut clientId=NEW_CONNECTION+1;

//     loop {
//         let event_cnt=match epoll_wait(epoll_fd, &mut events, -1) {
//             Ok(v)=>v,
//             Err(err)=>panic!("epoll_wait() error"),
//         };

//         for event in &events[0..event_cnt] {
//             match event.data() {
//                 NEW_CONNECTION=>{
//                     match listener.accept(){
//                         Ok((stream,_))=>{
//                             let event=EpollEvent::new(EpollFlags::EPOLLIN|EpollFlags::EPOLLRDHUP, clientId);
//                             epoll_ctl(epoll_fd, EpollOp::EpollCtlAdd, stream.as_raw_fd(), &mut event);
//                             clients.insert(clientId, stream);
//                             clientId+=1;
//                         }
//                         Err(err)=>eprintln!("accept err: {:?}",err),
//                     }
//                 }
//                 _=>{
//                     // this event is readable
//                     if event.events().contains(EpollFlags::EPOLLIN){
//                         let stream=clients.get(&event.data()).unwrap();
//                         let decoder=MQTinyDecoder{};
//                         FramedRead::new(stream, decoder);
//                     }
//                 }
//             }
//         }
//     }
// }

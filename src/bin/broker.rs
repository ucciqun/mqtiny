use bytes::Buf;
use clap::Parser;
use tokio::net::TcpListener;
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, FramedRead};

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
struct MQTinyDecoder {}
impl Decoder for MQTinyDecoder {
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

        src.advance(2 + remaining_length);

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
                    payload: "test".to_string(),
                };
                Ok(Some(p))
            }
            _ => {
                println!("hge");
                Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "oh"))
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let listener = TcpListener::bind(&format!("127.0.0.1:{}", args.port))
        .await
        .unwrap();
    println!("Listening on 127.0.0.1:{}", args.port);

    loop {
        let (client, _) = listener.accept().await.unwrap();

        println!("nankakita");

        let codec = MQTinyDecoder {};
        let mut frame_reader = FramedRead::new(client, codec);

        while let Some(frame) = frame_reader.next().await {
            match frame {
                Ok(data) => {
                    println!("{:?}", data.payload);
                }
                Err(err) => eprintln!("error: {:?}", err),
            }
        }
    }
}

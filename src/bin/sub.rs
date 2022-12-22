use bytes::{Buf, BytesMut};
use clap::Parser;
use std::str;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
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

#[derive(Parser, Debug)]
#[command(name = "MQTiny", author = "Ryo OUCHI")]
struct Args {
    /// Address of the MQTiny server to connect
    #[arg(short, long)]
    ip: String,

    /// MQTiny service port
    #[arg(short, long, default_value_t = 1883)]
    port: u16,

    /// Subscrived topics
    #[arg(short, long)]
    topic: u16,
}

#[allow(unused)]
struct PublishPacket {
    packet_type: u8,
    remaining_length: u8,
    topic_name: u16,
    payload: String,
}
struct MQTinyPublishDecoder {}
impl Decoder for MQTinyPublishDecoder {
    type Item = PublishPacket;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }

        let packet_type = src[0] >> 4;
        let remaining_length = src[1] as usize;

        if src.len() < (2 + remaining_length).into() {
            src.reserve(2 + remaining_length - src.len());

            return Ok(None);
        }

        let mut topic_name_bytes = [0u8; 2];
        topic_name_bytes.copy_from_slice(&src[2..4]);
        let topic_name = u16::from_be_bytes(topic_name_bytes);

        let data = src[(2 + 2)..(2 + remaining_length)].to_vec();
        src.advance(2 + remaining_length);

        match String::from_utf8(data) {
            Ok(payload) => {
                let p = PublishPacket {
                    packet_type,
                    remaining_length: remaining_length.try_into().unwrap(),
                    topic_name,
                    payload,
                };
                Ok(Some(p))
            }
            Err(utf8_error) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                utf8_error.utf8_error(),
            )),
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut stream = TcpStream::connect(&format!("{}:{}", args.ip, args.port))
        .await
        .unwrap();
    println!("Connecting on {}:{}", args.ip, args.port);

    //
    // Send Subscrive packet
    //

    let mut request = Vec::new();
    let packet_type = PacketType::Subscribe;
    let topic_length = 2;
    let total_length = topic_length;

    request.push(((packet_type as u8) << 4) as u8);
    request.push(total_length as u8);
    request.push((args.topic >> 8) as u8);
    request.push((args.topic & 0xff) as u8);
    // request.push(0 as u8); // padding
    // request.push(0 as u8); // padding

    stream.write_all(&request).await.unwrap();
    request.clear();

    let codec = MQTinyPublishDecoder {};
    let (r, _) = stream.split();
    let mut frame_reader = FramedRead::new(r, codec);

    let mut count = 0;
    while let Some(frame) = frame_reader.next().await {
        match frame {
            Ok(_data) => {
                // println!("{}", data.payload);
                count += 1;
                if count % 1000 == 0 {
                    println!("{}", count);
                };
            }
            Err(err) => eprintln!("error: {:?}", err),
        }
    }
}

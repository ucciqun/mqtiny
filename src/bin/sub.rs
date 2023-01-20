use bytes::BufMut;
use clap::Parser;
use mqtiny::*;
use std::str;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::FramedRead;

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
    request.put_u16(args.topic);
    // request.push(0 as u8); // padding
    // request.push(0 as u8); // padding

    stream.write_all(&request).await.unwrap();
    request.clear();

    let codec = MQTinyCodec {};
    let (r, _) = stream.split();
    let mut frame_reader = FramedRead::new(r, codec);

    let mut count = 0;
    while let Some(frame) = frame_reader.next().await {
        match frame {
            Ok(data) => {
                match data {
                    MqttPacket::Publish(MqttPublishPacket {
                        topic_name: _,
                        qos: _,
                        payload: _,
                    }) => {}
                    MqttPacket::Subscribe(_) => todo!(),
                    MqttPacket::Puback(_) => todo!(),
                }
                count += 1;
                if count % 1000 == 0 {
                    println!("{}", count);
                };
            }
            Err(err) => eprintln!("error: {:?}", err),
        }
    }
}

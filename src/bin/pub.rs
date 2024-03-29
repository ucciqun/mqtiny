use clap::Parser;
use std::{error::Error, thread};
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    time::{Duration, Instant},
};

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

    /// Total number of clients
    #[arg(short, long, default_value_t = 200)]
    count: usize,

    /// Interval to publish a message
    #[arg(short = 'I', long, default_value_t = 1000)]
    interval_of_msg: u64,

    /// Published topics
    #[arg(short, long)]
    topic: u16,

    /// Message Payload size (bytes)
    #[arg(short, long, default_value_t = 10)]
    size: u8,

    /// Number of messages to publish
    #[arg(short, long, default_value_t = 5000)]
    messages: u32,

    /// QoS level
    #[arg(short, long, default_value_t = 0)]
    qos: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut connections = Vec::new();
    let mut handles = Vec::new();

    for _ in 0..args.count {
        let stream = TcpStream::connect(&format!("{}:{}", args.ip, args.port))
            .await
            .unwrap();
        connections.push(stream);
    }

    println!("start publishing");

    loop {
        if let Some(mut stream) = connections.pop() {
            handles.push(tokio::spawn(async move {
                let start = Instant::now();

                let mut request = Vec::new();
                let packet_type = PacketType::Publish; // [7:4] in fixed header
                let topic_length = 2;
                let total_length = args.size + topic_length;
                let payload = "A".repeat(args.size.into());

                request.push(((packet_type as u8) << 4) as u8 + (args.qos << 1) as u8);
                request.push(total_length as u8);
                request.push((args.topic >> 8) as u8);
                request.push((args.topic & 0xff) as u8);
                request.extend_from_slice(payload.as_bytes());

                // println!("{:?}", request);

                let mut count = 0;

                stream.write_all(&request).await.unwrap();
                thread::sleep(Duration::from_millis(1000));
                count += 1;

                for _ in 0..args.messages - 1 {
                    //
                    // Send Publish packet
                    //
                    stream.write_all(&request).await.unwrap();

                    if args.interval_of_msg != 0 {
                        thread::sleep(Duration::from_millis(args.interval_of_msg));
                    }

                    count += 1;
                }

                println!("published {} messages", count);

                let elapsed = start.elapsed();
                println!("{:?}", elapsed);
            }));
        } else {
            break;
        }
    }

    futures::future::join_all(handles).await;

    Ok(())
}

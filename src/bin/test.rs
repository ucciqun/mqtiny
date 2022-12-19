use std::{thread, time::Duration};

fn main() {
    // let v = vec![1, 2, 3];

    // let handle = thread::spawn(move || {
    //     println!("Here's a vector: {:?}", v);
    // });
    // handle.join().unwrap();

    // for i in 1..5 {
    //     println!("hi number {} from the main thread!", i);
    //     thread::sleep(Duration::from_millis(1));
    // }

    let payload = "A".repeat(10);
    let mut request = [0; 1024];
    request[4..].copy_from_slice(&payload.as_bytes());
    println!("{:?}", payload.as_bytes());
    println!("{:?}", request);
}

// use clap::Parser;
// use std::io;
// use std::io::prelude::*;
// use std::net::TcpStream;
// use std::time::{Duration, Instant};
// use tokio::stream;

// #[allow(unused)]
// #[derive(Debug)]
// enum PacketType {
//     Unknown = 0,
//     Connect = 1,
//     Connack = 2,
//     Publish = 3,
//     Puback = 4,
//     Pubrec = 5,
//     Pubrel = 6,
//     Pubcomp = 7,
//     Subscribe = 8,
//     Suback = 9,
//     Unsubscribe = 10,
//     Unsuback = 11,
//     Pingreq = 12,
//     Pingresp = 13,
//     Disconnect = 14,
//     Reserved = 15,
// }

// #[derive(Parser, Debug)]
// #[command(name = "MQTiny", author = "Ryo OUCHI")]
// struct Args {
//     /// Address of the MQTiny server to connect
//     #[arg(short, long)]
//     ip: String,

//     /// MQTiny service port
//     #[arg(short, long, default_value_t = 1883)]
//     port: u16,

//     /// Total number of clients
//     #[arg(short, long, default_value_t = 200)]
//     count: u16,

//     /// Interval to publish a message
//     #[arg(short = 'I', long, default_value_t = 1000)]
//     interval_of_msg: u64,

//     /// Published topics
//     #[arg(short, long)]
//     topic: u16,

//     /// Message Payload size (bytes)
//     #[arg(short, long, default_value_t = 10)]
//     size: u8,

//     /// Number of messages to publish
//     #[arg(short, long, default_value_t = 5000)]
//     messages: u32,
// }

// pub fn main() -> io::Result<()> {
//     let args = Args::parse();
//     let mut connections = Vec::new();

//     //
//     // Connect
//     for _ in 0..args.count {
//         let stream = TcpStream::connect(&format!("{}:{}", args.ip, args.port))?;
//         connections.push(stream);
//     }

//     //
//     // Create Publish packet
//     //
//     let mut request = Vec::new();
//     let packet_type = PacketType::Publish;
//     let topic_length = 2;
//     let total_length = args.size + topic_length;
//     let payload = "A".repeat(args.size.into());
//     request.push(packet_type as u8);
//     request.push(total_length as u8);
//     request.push((args.topic >> 8) as u8);
//     request.push((args.topic & 0xff) as u8);
//     request.extend_from_slice(payload.as_bytes());

//     let mut count = 0;
//     let start = Instant::now();

//     for _ in 0..args.messages {
//         //
//         // Send Publish packet
//         //
//         stream.write_all(&request).unwrap();

//         //
//         // Receive Puback packet
//         //
//         let mut response = [0; 1024];
//         let _n = stream.read(&mut response)?;

//         let response_packet_type = response[0];
//         if response_packet_type == PacketType::Puback as u8 {
//             // println!("PUBACK received");
//         }

//         count += 1;

//         std::thread::sleep(Duration::from_millis(args.interval_of_msg));
//     }
//     stream.shutdown(std::net::Shutdown::Both)?;

//     let elapsed = start.elapsed();

//     print!("{:?},", elapsed.as_micros());
//     println!();
//     println!("published messages: {}", count);

//     Ok(())
// }

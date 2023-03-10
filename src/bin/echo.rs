use clap::Parser;
use std::{
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Parser, Debug)]
#[command(name = "MQTiny", author = "Ryo OUCHI")]
struct Args {
    /// Address of the MQTiny server to connect
    #[arg(short, long)]
    ip: String,

    /// MQTiny service port
    #[arg(short, long, default_value_t = 1883)]
    port: u16,

    /// Interval to publish a message
    #[arg(short = 'I', long, default_value_t = 1000)]
    interval_of_msg: u64,

    /// Number of messages to publish
    #[arg(short, long, default_value_t = 5000)]
    messages: u32,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let mut stream = TcpStream::connect(&format!("{}:{}", args.ip, args.port))
        .await
        .unwrap();

    for _ in 0..args.messages {
        let mut request = Vec::new();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        request.write_u128(now.as_nanos()).await.unwrap();
        stream.write_all(&request).await.unwrap();

        let a = stream.read_u128().await.unwrap();
        let b = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        println!("{},", b - a);

        thread::sleep(Duration::from_micros(args.interval_of_msg));
    }
}

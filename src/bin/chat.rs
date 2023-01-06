use std::{collections::HashMap, env, error::Error, io, net::SocketAddr, sync::Arc};

use futures::SinkExt;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let state = Arc::new(Mutex::new(Shared::new()));

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6142".to_string());
    let listener = TcpListener::bind(&addr).await?;

    loop {
        let (stream, addr) = listener.accept().await?;
        let state = Arc::clone(&state);

        tokio::spawn(async move {
            if let Err(e) = process(state, stream, addr).await {
                eprintln!("{:?}", e);
            }
        });
    }
}

type Tx = mpsc::UnboundedSender<String>;
type Rx = mpsc::UnboundedReceiver<String>;

struct Shared {
    peers: HashMap<SocketAddr, Tx>,
}

struct Peer {
    lines: Framed<TcpStream, LinesCodec>,
    rx: Rx,
}

impl Shared {
    fn new() -> Self {
        Shared {
            peers: HashMap::new(),
        }
    }

    async fn broadcast(&mut self, sender: SocketAddr, message: &str) {
        for peer in self.peers.iter_mut() {
            if *peer.0 != sender {
                let _ = peer.1.send(message.into());
            }
        }
    }
}

impl Peer {
    async fn new(
        state: Arc<Mutex<Shared>>,
        lines: Framed<TcpStream, LinesCodec>,
    ) -> io::Result<Peer> {
        let addr = lines.get_ref().peer_addr()?;
        let (tx, rx) = mpsc::unbounded_channel();
        state.lock().await.peers.insert(addr, tx);
        Ok(Peer { lines, rx })
    }
}

async fn process(
    state: Arc<Mutex<Shared>>,
    stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let mut lines = Framed::new(stream, LinesCodec::new());
    lines.send("Please enter your username:").await?;
    let username = match lines.next().await {
        Some(Ok(line)) => line,
        _ => return Ok(()),
    };
    let mut peer = Peer::new(state.clone(), lines).await?;
    {
        let mut state = state.lock().await;
        let msg = format!("{} has joined the chat", username);
        state.broadcast(addr, &msg).await;
    }

    loop {
        tokio::select! {
            Some(msg)=peer.rx.recv()=>{
                peer.lines.send(&msg).await?;
            }
            result=peer.lines.next()=>match result{
                Some(Ok(msg))=>{
                    let mut state=state.lock().await;
                    let msg=format!("{}:{}",username,msg);
                    state.broadcast(addr, &msg).await;
                }
                Some(Err(_e))=>{},
                None=>break,
            }
        }
    }

    {
        let mut state = state.lock().await;
        state.peers.remove(&addr);

        let msg = format!("{} has left the chat", username);
        state.broadcast(addr, &msg).await;
    }
    Ok(())
}

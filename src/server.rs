use std::{
    io,
    net::SocketAddr,
    sync::{Arc, RwLock},
    time::Instant,
};

use bevy_ecs::{entity::Entity, world::World};
use dashmap::DashMap;
use serde::Serialize;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::mpsc::{self, Receiver, Sender},
};
use uuid::Uuid;

use crate::{
    event::{handle_event, EventContext, State},
    protocol::{
        packets::{Header, PluginMessage},
        serde::{deserialize_from_slice, serialize_to_vec, serialize_with_size, v32, VarInt},
    },
};

pub type PeersMap = Arc<DashMap<SocketAddr, Sender<Arc<[u8]>>>>;

// TODO: move this to mod.rs, actually to server.rs
pub struct Server {
    socket: TcpListener,
}

impl Server {
    /// Creates a new Server, which is utility struct used for accepting new connections.
    /// # PANICS
    /// This function panics whenever socket fails to bind on specified port.
    pub async fn new() -> Self {
        Self {
            socket: TcpListener::bind("0.0.0.0:25566")
                .await
                .expect("Couldn't bind on port 25565"),
        }
    }

    pub async fn run(self) -> ! {
        let peers = Arc::new(DashMap::new());

        let xd = PluginMessage::<Vec<u8>> {
            channel: String::from("MC|Brand"),
            data: "troad".to_owned().into_bytes(),
        };

        println!("{:02x?}", serialize_to_vec(&xd).unwrap());
        println!("{:02x?}", bincode::serialize(&xd).unwrap());
        println!("{}", Uuid::new_v4().to_string());

        let world = Arc::new(RwLock::new(World::new()));

        loop {
            match self.socket.accept().await {
                Ok((stream, addr)) => {
                    let (tx, rx) = mpsc::channel::<Arc<[u8]>>(100);

                    let peers = peers.clone();
                    peers.insert(addr, tx.clone());

                    // Yes, this should panic if it fails bro.
                    stream.set_nodelay(true).unwrap();

                    let world = world.clone();

                    tokio::spawn(async move {
                        if let Err(e) =
                            Server::handle_connection(peers.clone(), addr, stream, rx, world).await
                        {
                            eprintln!("Error occurred while trying to handle connection {e}");
                        }

                        // Dropping the channel should make it disconnect.
                        peers.remove(&addr).unwrap();
                    });
                }
                Err(e) => self.handle_io_error(e).await,
            }
        }
    }

    async fn handle_io_error(&self, e: io::Error) {
        match e.kind() {
            _ => eprintln!("Accepting a connection has failed. Unhandled error: {e}"),
        }
    }

    async fn handle_connection(
        peers: PeersMap,
        addr: SocketAddr,
        mut stream: TcpStream,
        mut rx: Receiver<Arc<[u8]>>,
        world: Arc<RwLock<World>>,
    ) -> Result<(), io::Error> {
        // Decently-sized receive buffer.
        // Each connected client would at least take in 256 KiB...
        // 100 would require roughly ~4 MiB.
        // TODO: to improve perf, this could be stored in a stack...?
        let mut buf = vec![0; 1024 * 256];
        let mut state = State::Handshaking;
        let mut entity = None;

        loop {
            tokio::select! {
                res = stream.read(&mut buf[..]) => {
                    let size = res?;
                    if size == 0 {
                        return Ok(());
                    }

                    // println!("Received {size} from {addr}: {:02x?}", &buf[..size]);
                    let start = Instant::now();
                    handle_incoming_data(&peers, addr, &mut stream, &mut state, &buf[..size], world.clone(), &mut entity).await?;
                    let end = Instant::now();

                    if (end - start).as_micros() > 300 {
                        println!("Took {}ms/{}μs/{}ns!", (end - start).as_millis(), (end - start).as_micros(), (end - start).as_nanos());
                    }
                }
                msg = rx.recv() => {
                    if let Some(msg) = msg {
                        stream.write_all(&*msg).await?;
                    } else {
                        return Ok(());
                    }
                }
            }
        }
    }
}

async fn handle_incoming_data(
    peers: &PeersMap,
    addr: SocketAddr,
    stream: &mut TcpStream,
    state: &mut State,
    buf: &[u8],
    world: Arc<RwLock<World>>,
    entity: &mut Option<Entity>,
) -> Result<(), io::Error> {
    let mut read = 0;
    while read < buf.len() {
        let (size, header) = deserialize_from_slice::<Header>(&buf[read..])?;

        handle_event(EventContext {
            peers,
            state,
            stream,
            buf: &buf[read + size..],
            header,
            world: world.clone(),
            entity,
        })
        .await?;
        read += size + *header.len as usize - 1; // FIXME: this doesn't work if the packet id is too large.
    }

    Ok(())
}

// TODO: need a better way. so for now we just IGNORE.
async fn _handle_if_lslp(socket: &mut TcpStream, buf: &[u8]) -> bool {
    if buf[0..2] == [0xFE, 0x01, 0xFA] {
        println!("A legacy SLP!");

        let mut vec: Vec<u8> = Vec::with_capacity(128);
        vec.push(0xFF);

        let response = format!(
            "§1\0{}\0{}\0{}\0{}\0{}",
            127,
            "Troad 1.8.9",
            "Incompatible with other versions.", // TODO: display real MOTD?
            0,                                   // cur players
            0                                    // max players
        );

        let len = (response.len() as u16 - 1).to_be_bytes();
        vec.push(len[0]);
        vec.push(len[1]);

        let utf16 = response
            .encode_utf16()
            .map(|n| u16::to_be_bytes(n))
            .flatten();
        vec.extend(utf16);

        socket.write_all(&vec[..]).await.unwrap();
        socket.shutdown().await.unwrap();

        true
    } else {
        false
    }
}

pub trait TcpProtocolExt {
    async fn send<T: Serialize>(&mut self, id: i32, p: &T) -> io::Result<()>;
}

impl TcpProtocolExt for TcpStream {
    async fn send<T: Serialize>(&mut self, id: i32, p: &T) -> io::Result<()> {
        #[derive(Serialize)]
        pub struct Data<'a, T> {
            id: v32,
            p: &'a T,
        }

        self.write_all(&serialize_with_size(&Data {
            id: VarInt::<i32>(id),
            p,
        })?)
        .await
    }
}

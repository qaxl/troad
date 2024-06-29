// use std::{
//     io,
//     net::SocketAddr,
//     sync::{Arc, RwLock},
//     time::Instant,
// };

// use bevy_ecs::{change_detection::MutUntyped, entity::Entity, world::World};
// use cipher::{
//     generic_array::sequence::GenericSequence, inout::InOutBuf, BlockDecryptMut, BlockEncryptMut,
//     KeyInit, KeyIvInit,
// };
// use dashmap::DashMap;
// use rsa::{pkcs1::EncodeRsaPublicKey, pkcs8::Document, RsaPrivateKey, RsaPublicKey};
// use serde::Serialize;
// use tokio::{
//     io::{AsyncReadExt, AsyncWriteExt},
//     net::{TcpListener, TcpStream},
//     sync::mpsc::{self, Receiver, Sender},
// };
// use troad_serde::{de::from_slice, ser::to_vec_with_size,};

// use crate::{
//     event::{handle_event, Aes128Cfb8Dec, Aes128Cfb8Enc, Encryption, EventContext, State},
// };

// pub type PeersMap = Arc<DashMap<SocketAddr, Sender<Arc<[u8]>>>>;

// // TODO: move this to mod.rs, actually to server.rs
// pub struct Server {
//     socket: TcpListener,
// }

// impl Server {
//     /// Creates a new Server, which is utility struct used for accepting new connections.
//     /// # PANICS
//     /// This function panics whenever socket fails to bind on specified port.
//     pub async fn new() -> Self {
//         Self {
//             socket: TcpListener::bind("0.0.0.0:25566")
//                 .await
//                 .expect("Couldn't bind on port 25565"),
//         }
//     }

//     pub async fn run(self) -> ! {
//         let peers = Arc::new(DashMap::new());

//         let world = Arc::new(RwLock::new(World::new()));

//         loop {
//             match self.socket.accept().await {
//                 Ok((stream, addr)) => {
//                     let (tx, rx) = mpsc::channel::<Arc<[u8]>>(100);

//                     let peers = peers.clone();
//                     peers.insert(addr, tx.clone());

//                     // Yes, this should panic if it fails bro.
//                     stream.set_nodelay(true).unwrap();

//                     let world = world.clone();

//                     tokio::spawn(async move {
//                         if let Err(e) =
//                             Server::handle_connection(peers.clone(), addr, stream, rx, world).await
//                         {
//                             eprintln!("Error occurred while trying to handle connection {e}");
//                         }

//                         // Dropping the channel should make it disconnect.
//                         peers.remove(&addr).unwrap();
//                     });
//                 }
//                 Err(e) => self.handle_io_error(e).await,
//             }
//         }
//     }

//     async fn handle_io_error(&self, e: io::Error) {
//         match e.kind() {
//             _ => eprintln!("Accepting a connection has failed. Unhandled error: {e}"),
//         }
//     }

//     async fn handle_connection(
//         peers: PeersMap,
//         addr: SocketAddr,
//         mut stream: TcpStream,
//         mut rx: Receiver<Arc<[u8]>>,
//         world: Arc<RwLock<World>>,
//     ) -> Result<(), io::Error> {
//         // Decently-sized receive buffer.
//         // Each connected client would at least take in 256 KiB...
//         // 100 would require roughly ~4 MiB.
//         // TODO: to improve perf, this could be stored in a stack...?
//         let mut buf = vec![0; 1024 * 256];
//         let mut state = State::Handshaking;
//         let mut entity = None;

//         let priv_key = {
//             let mut rng = rand::thread_rng();
//             let bits = 1024;
//             RsaPrivateKey::new(&mut rng, bits).expect("Failed to generate a key")
//         };

//         let pub_key = RsaPublicKey::from(&priv_key);

//         let mut verify_token = rand::random::<u128>();
//         let mut shared_secret = Vec::new();
//         let mut cipher = None;

//         loop {
//             tokio::select! {
//                 res = stream.read(&mut buf[..]) => {
//                     let size = res?;
//                     if size == 0 {
//                         return Ok(());
//                     }

//                     // println!("Received {size} from {addr}: {:02x?}", &buf[..size]);
//                     let start = Instant::now();
//                     handle_incoming_data(&peers, addr, &mut stream, &mut state, &mut buf[..size], world.clone(), &mut entity, &mut Encryption { pub_key: pub_key.clone(), priv_key: priv_key.clone() }, verify_token, &mut shared_secret, &mut cipher).await?;
//                     let end = Instant::now();

//                     if (end - start).as_micros() > 300 {
//                         println!("Took {}ms/{}μs/{}ns!", (end - start).as_millis(), (end - start).as_micros(), (end - start).as_nanos());
//                     }
//                 }
//                 msg = rx.recv() => {
//                     if let Some(msg) = msg {
//                         stream.write_all(&*msg).await?;
//                     } else {
//                         return Ok(());
//                     }
//                 }
//             }
//         }
//     }
// }

// async fn handle_incoming_data(
//     peers: &PeersMap,
//     addr: SocketAddr,
//     stream: &mut TcpStream,
//     state: &mut State,
//     buf: &mut [u8],
//     world: Arc<RwLock<World>>,
//     entity: &mut Option<Entity>,
//     pub_key: &mut Encryption,
//     verify_token: u128,
//     shared_secret: &mut Vec<u8>,
//     cipher: &mut Option<Cipher>,
// ) -> Result<(), io::Error> {
//     let mut read = 0;
//     if let Some(cipher) = cipher {
//         // encryption has been enabled bro
//         // no idea will this work tho...
//         decrypt_packet(&mut cipher.1, buf);
//         // Aes128Cfb8Dec::new((&shared_secret[..]).into(), (&shared_secret[..]).into()).decrypt(buf);
//     }

//     println!(
//         "Received a (encrypted: {}) {}-byte packet.",
//         shared_secret.len() != 0,
//         buf.len()
//     );
//     while read < buf.len() {
//         let (size, header) = from_slice::<Header>(&buf[read..])?;

//         handle_event(EventContext {
//             peers,
//             state,
//             stream,
//             buf: &buf[read + size..],
//             header,
//             world: world.clone(),
//             entity,
//             addr: &addr,
//             encryption: pub_key,
//             verify_token,
//             shared_secret,
//             cipher,
//         })
//         .await?;
//         read += size + *header.len as usize - 1; // FIXME: this doesn't work if the packet id is too large.
//     }

//     Ok(())
// }

// pub fn decrypt_packet(cipher: &mut Aes128Cfb8Dec, packet: &mut [u8]) {
//     let (chunks, rest) = InOutBuf::from(packet).into_chunks();
//     assert!(rest.is_empty());
//     cipher.decrypt_blocks_inout_mut(chunks);
// }

// // TODO: need a better way. so for now we just IGNORE.
// async fn _handle_if_lslp(socket: &mut TcpStream, buf: &[u8]) -> bool {
//     if buf[0..2] == [0xFE, 0x01, 0xFA] {
//         println!("A legacy SLP!");

//         let mut vec: Vec<u8> = Vec::with_capacity(128);
//         vec.push(0xFF);

//         let response = format!(
//             "§1\0{}\0{}\0{}\0{}\0{}",
//             127,
//             "Troad 1.8.9",
//             "Incompatible with other versions.", // TODO: display real MOTD?
//             0,                                   // cur players
//             0                                    // max players
//         );

//         let len = (response.len() as u16 - 1).to_be_bytes();
//         vec.push(len[0]);
//         vec.push(len[1]);

//         let utf16 = response
//             .encode_utf16()
//             .map(|n| u16::to_be_bytes(n))
//             .flatten();
//         vec.extend(utf16);

//         socket.write_all(&vec[..]).await.unwrap();
//         socket.shutdown().await.unwrap();

//         true
//     } else {
//         false
//     }
// }

// pub struct Cipher(Aes128Cfb8Enc, Aes128Cfb8Dec);

// impl Cipher {
//     pub fn new(key: &[u8]) -> Cipher {
//         Cipher(
//             Aes128Cfb8Enc::new_from_slices(key, key).unwrap(),
//             Aes128Cfb8Dec::new_from_slices(key, key).unwrap(),
//         )
//     }
// }

// pub trait TcpProtocolExt {
//     async fn send<T: Serialize>(&mut self, id: i32, p: &T) -> io::Result<()>;
//     async fn send_enc<T: Serialize>(
//         &mut self,
//         cipher: &mut Cipher,
//         id: i32,
//         p: &T,
//     ) -> io::Result<()>;
// }

// impl TcpProtocolExt for TcpStream {
//     async fn send<T: Serialize>(&mut self, id: i32, p: &T) -> io::Result<()> {
//         #[derive(Serialize)]
//         pub struct Data<'a, T> {
//             id: VarInt,
//             p: &'a T,
//         }

//         self.write_all(&to_vec_with_size(&Data { id: VarInt(id), p })?)
//             .await
//     }

//     async fn send_enc<T: Serialize>(
//         &mut self,
//         cipher: &mut Cipher,
//         id: i32,
//         p: &T,
//     ) -> io::Result<()> {
//         #[derive(Serialize)]
//         pub struct Data<'a, T> {
//             id: VarInt,
//             p: &'a T,
//         }

//         let mut p = to_vec_with_size(&Data { id: VarInt(id), p })?;
//         {
//             let (chunks, leftovers) = InOutBuf::from(&mut p[..]).into_chunks();
//             if !leftovers.is_empty() {
//                 panic!("what the fuck");
//             }

//             cipher.0.encrypt_blocks_inout_mut(chunks);
//         }
//         self.write_all(&p).await
//     }
// }

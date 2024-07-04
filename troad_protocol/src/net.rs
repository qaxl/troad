use std::{
    borrow::Cow,
    io::{self, ErrorKind, Read, Write},
    ops::{Deref, Range},
};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use troad_crypto::conn;
use troad_serde::{
    from_slice,
    tinyvec::{tiny_vec, TinyVec},
    to_vec, to_vec_with_size, var_int,
};

use crate::login::{client_bound::SetCompression, ClientBound};

pub struct Connection {
    conn: conn::Connection,

    discarded_data: TinyVec<[u8; 128]>,
    compression_threshold: Option<usize>,
}

#[derive(Serialize, Deserialize)]
struct CompressedPacket<'a> {
    #[serde(with = "var_int")]
    uncompressed_len: usize,
    compressed_data: Cow<'a, [u8]>,
}

#[derive(Deserialize)]
pub struct PacketSize(#[serde(with = "var_int")] usize);

impl Deref for PacketSize {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Connection {
    pub async fn send<P: Serialize>(&mut self, p: &P) -> Result<(), io::Error> {
        let mut p = if let Some(compression_threshold) = self.compression_threshold {
            let p = to_vec(p)?;
            let len = p.len();

            let uncompressed_len;
            let p = if len > compression_threshold {
                // Compress packet id + data
                let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
                // FIXME: are ALL packet types 1 byte?
                e.write_all(&p[0..1])?;
                e.write_all(&p[1..])?;

                uncompressed_len = len;
                &e.finish()?[..]
            } else {
                uncompressed_len = 0;
                &p[..]
            };

            println!("{uncompressed_len}");

            // Construct a compressed packet header
            // current len + compressed len + compressed data
            to_vec_with_size(&CompressedPacket {
                uncompressed_len,
                compressed_data: Cow::Borrowed(p),
            })?
        } else {
            to_vec_with_size(p)?
        };

        self.conn.send(&mut p[..]).await
    }

    pub async fn recv<P: for<'a> Deserialize<'a>>(&mut self) -> Result<P, io::Error> {
        let mut buf = TinyVec::<[u8; 128]>::from_array_len([0; 128], 128);
        if self.discarded_data.len() > 0 {
            println!("B");
            let mut buf = self.discarded_data.clone();
            let read = self._recv(&mut buf).await?;

            Ok(from_slice(&buf[read..])?.1)
        } else {
            // println!("A");
            let recv = self.conn.recv(&mut buf).await?;
            let mut buf = TinyVec::from(&buf[..recv]);
            let read = self._recv(&mut buf).await?;

            println!("{buf:02x?} {}", buf.len());
            Ok(from_slice(&buf[read..])?.1)
        }
    }

    async fn _recv(&mut self, buf: &mut TinyVec<[u8; 128]>) -> Result<usize, io::Error> {
        let (read, size) = from_slice::<PacketSize>(&buf)?;
        let total = read + *size;

        println!("Receiving data: {total} {}", buf.len());

        if total > buf.len() {
            let mut recv = buf.len();
            buf.resize(total, 0);

            println!("{} {total} {buf:02x?}", buf.len());
            while recv != total {
                println!("{recv} {total}");
                recv += self.conn.recv(&mut buf[recv..]).await?;
            }

            self.discarded_data.clear();
        } else if total < buf.len() {
            // This doesn't actually "override", because if there's already existing data, it has been passed to this function as `buf`
            // and is therefore handled already.
            println!("Discarding {}", buf.len() - total);
            self.discarded_data = TinyVec::from(&buf[total..]);
        } else {
            // Reset it otherwise.
            self.discarded_data.clear();
        }

        println!("{buf:02x?}");
        Ok(read)
    }

    // TODO: custom error type
    pub fn enable_encryption(&mut self, shared_secret: &[u8]) -> Result<(), io::Error> {
        if let Err(_) = self.conn.enable_encryption(shared_secret) {
            Err(io::Error::other("shared_secret is invalid length"))
        } else {
            Ok(())
        }
    }

    // This function enables compression and sends information about it to the client.
    // You don't manually need to send SetCompression (0x3, login) packet.
    // TODO: should this send it? ^
    pub async fn set_compression(&mut self, threshold: Option<usize>) -> Result<(), io::Error> {
        if let Some(threshold) = threshold {
            self.send(&ClientBound::SetCompression(SetCompression { threshold }))
                .await?;
        }
        self.compression_threshold = threshold;

        Ok(())
    }
}

impl From<TcpStream> for Connection {
    fn from(value: TcpStream) -> Self {
        value.set_nodelay(true).unwrap();
        Self {
            conn: conn::Connection::from(value),
            compression_threshold: None,
            discarded_data: TinyVec::new(),
        }
    }
}

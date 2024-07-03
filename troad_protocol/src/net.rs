use std::{
    borrow::Cow,
    io::{self, ErrorKind, Read, Write},
    ops::{Deref, Range},
};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use troad_crypto::conn;
use troad_serde::{from_slice, tinyvec::{tiny_vec, TinyVec}, to_vec, to_vec_with_size, var_int};

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

        let (mut buf, recv, sz) = if self.discarded_data.len() > 0 {
            // try see if there's already enough data
            match from_slice::<PacketSize>(&self.discarded_data) {
                Ok((read, size)) => {
                    let data = self.discarded_data.clone();
                    let mut total_recv = self.discarded_data.len();

                    if (read + *size) > self.discarded_data.len() {
                        // need more data
                        self.discarded_data.resize(read + *size, 0);
                        while total_recv != (read + *size) {
                            total_recv += self.conn.recv(&mut self.discarded_data[total_recv..]).await?;
                        }
                    } else if (read + *size) < self.discarded_data.len() {
                        // lol still too much data, but eh. we'll take it
                        self.discarded_data = TinyVec::from(&data[read + *size..]);
                    } else {
                        self.discarded_data = TinyVec::new();
                    }

                    (self.discarded_data.clone(), total_recv, Some((read, size)))
                },
                Err(_) => /* most likely not enough data */ (buf, 0, None),
            }
        } else {
            let recv = self.conn.recv(&mut buf[..]).await?;
            if recv == 0 {
                return Err(io::Error::other("peer disconnected gracefully"));
            }

            (buf, recv, None)
        };

        if let Some(threshold) = self.compression_threshold {
            let (read, size) = from_slice::<[PacketSize; 2]>(&buf)?;

            let compressed_size = *size[0];
            let uncompressed_size = *size[1];

            println!("Compressed packet {{ compressed_size = {compressed_size}, uncompressed_size = {uncompressed_size}, is_over_threshold = {}, is_compressed = {} }}", threshold < compressed_size, compressed_size != 0);
            if compressed_size + read > recv {
                println!("\tReceived too little data!");
            } else if compressed_size + read < recv {
                println!("\tReceived too much data!");
            } else {
                println!("\tReceived just enough data!");
            }

            Ok(from_slice(&buf[read..])?.1)
        } else {
            let (read, size) = if let Some((read, size)) = sz {
                (read, size)
            } else {
                let (read, size) = from_slice::<PacketSize>(&buf)?;
                (read, size)
            };
            
            let mut total_recv = recv;

            println!("Uncompressed packet {{ size = {} }}", *size);
            if read + *size > recv {
                println!("\tReceived too little data!");

                // self.buf.resize(read + *size, 0);
                // let mut buf = []
                buf.resize(read + *size, 0);
                while total_recv != read + *size {
                    total_recv += self.conn.recv(&mut buf[total_recv..]).await?;
                }
            }

            let p = from_slice(&buf[read..])?;
            if read + *size < total_recv {
                println!("\tDiscarding {} bytes!", total_recv - read - *size);

                self.discarded_data = TinyVec::from(&buf[read + *size..]);
            }

            Ok(p.1)
        }
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

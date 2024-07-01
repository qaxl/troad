use std::{
    borrow::Cow,
    io::{self, ErrorKind, Read, Write},
    ops::Range,
};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use troad_crypto::conn;
use troad_serde::{from_slice, serde_bytes, to_vec, to_vec_with_size, var_int};

pub struct Connection {
    conn: conn::Connection,

    buf: Vec<u8>,
    excess_buf: Range<usize>,
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

impl Connection {
    pub async fn send<P: Serialize>(&mut self, p: &P) -> Result<(), io::Error> {
        let p = if let Some(compression_threshold) = self.compression_threshold {
            let p = to_vec(p)?;
            let len = p.len();

            let uncompressed_len;
            let p = if len > compression_threshold {
                // Compress packet id + data
                let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
                e.write_all(&p[..])?;

                uncompressed_len = len;
                &e.finish()?[..]
            } else {
                uncompressed_len = 0;
                &p[..]
            };

            // Construct a compressed packet header
            // current len + compressed len + compressed data
            &mut to_vec_with_size(&CompressedPacket {
                uncompressed_len,
                compressed_data: Cow::Borrowed(p),
            })?
        } else {
            &mut to_vec_with_size(p)?[..]
        };

        self.conn.send(p).await
    }

    async fn recv_more(
        &mut self,
        packet_size: Option<(usize, PacketSize)>,
    ) -> Result<(usize, usize, PacketSize), io::Error> {
        let mut size_ = self.conn.recv(&mut self.buf[..]).await?;
        let (read, packet_size) = if let Some(p) = packet_size {
            p
        } else {
            from_slice::<PacketSize>(&self.buf[..])?
        };

        if (size_ - read) < packet_size.0 {
            self.buf.resize(packet_size.0 + read, 0);

            let mut size = size_ - read;
            while size < packet_size.0 {
                let recv = self.conn.recv(&mut self.buf[(read + size)..]).await?;

                if recv == 0 {
                    return Err(io::Error::from(ErrorKind::UnexpectedEof));
                }

                size += recv;
            }

            size_ = size;
        }

        Ok((size_, read, packet_size))
    }

    pub async fn recv<P: for<'a> Deserialize<'a>>(&mut self) -> Result<P, io::Error> {
        if self.excess_buf.len() > 0 {
            match from_slice::<PacketSize>(&self.buf[self.excess_buf.clone()]) {
                Ok((read, packet_size)) => {
                    self.excess_buf = (self.excess_buf.start + read)..self.excess_buf.end;

                    if packet_size.0 > self.excess_buf.len() {
                        self.recv_more(Some((read, packet_size))).await?;
                    } else {
                        return Ok(from_slice::<P>(&self.buf[self.excess_buf.clone()])?.1);
                    }
                }
                Err(e) => {
                    eprintln!("a peer tried sending invalid packet length? e: {e}");
                    return Err(e.into());
                }
            }
        }

        let (size_, read, packet_size) = self.recv_more(None).await?;

        if let Some(compression_threshold) = self.compression_threshold {
            let (read_c, compressed_packet) = from_slice::<CompressedPacket>(&self.buf)?;
            self.excess_buf = (read + read_c)..size_;

            if packet_size.0 > compression_threshold {
                let mut d = ZlibDecoder::new(&compressed_packet.compressed_data[..]);
                let mut v = vec![0; compressed_packet.uncompressed_len];
                d.read_exact(&mut v[..])?;

                Ok(from_slice(&v[..])?.1)
            } else {
                Ok(from_slice(&self.buf[(read + read_c)..(packet_size.0 + read + read_c)])?.1)
            }
        } else {
            let (read_p, packet) = from_slice(&self.buf[read..packet_size.0 + read])?;
            self.excess_buf = (read + read_p)..size_;

            Ok(packet)
        }
    }
}

impl From<TcpStream> for Connection {
    fn from(value: TcpStream) -> Self {
        Self {
            conn: conn::Connection::from(value),
            excess_buf: 0..0,
            buf: vec![0; 1024],
            compression_threshold: None,
        }
    }
}

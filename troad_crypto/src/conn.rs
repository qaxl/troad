use std::io;

use aes::cipher::{inout::InOutBuf, BlockDecryptMut, BlockEncryptMut, InvalidLength, KeyIvInit};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

type Aes128Cfb8Encryptor = cfb8::Encryptor<aes::Aes128>;
type Aes128Cfb8Decryptor = cfb8::Decryptor<aes::Aes128>;

pub struct Connection {
    conn: TcpStream,
    encryption: Option<Encryption>,
}

impl Connection {
    pub fn enable_encryption(&mut self, shared_secret: &[u8]) -> Result<(), InvalidLength> {
        self.encryption = Some(Encryption::new(shared_secret)?);
        Ok(())
    }

    pub async fn send(&mut self, buf: &mut [u8]) -> Result<(), io::Error> {
        if let Some(encryption) = &mut self.encryption {
            let (blocks, tail) = InOutBuf::from(&mut *buf).into_chunks();
            assert!(tail.is_empty());

            encryption.encryptor.encrypt_blocks_inout_mut(blocks);
        }

        self.conn.write_all(buf).await
    }

    pub async fn recv(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        let read = self.conn.read(buf).await?;

        if read == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Connection was closed while reading data..."));
        }

        if let Some(encryption) = &mut self.encryption {
            let (blocks, tail) = InOutBuf::from(&mut *buf).into_chunks();
            assert!(tail.is_empty());

            encryption.decryptor.decrypt_blocks_inout_mut(blocks);
        }

        Ok(read)
    }
}

impl From<TcpStream> for Connection {
    fn from(conn: TcpStream) -> Self {
        Self {
            conn,
            encryption: None,
        }
    }
}

struct Encryption {
    encryptor: Aes128Cfb8Encryptor,
    decryptor: Aes128Cfb8Decryptor,
}

impl Encryption {
    pub fn new(shared_secret: &[u8]) -> Result<Self, InvalidLength> {
        Ok(Self {
            encryptor: Aes128Cfb8Encryptor::new_from_slices(shared_secret, shared_secret)?,
            decryptor: Aes128Cfb8Decryptor::new_from_slices(shared_secret, shared_secret)?,
        })
    }
}

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use rsa::{
    pkcs8::{der::Encode, SubjectPublicKeyInfo},
    Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey,
};

pub type RsaKey = (RsaPublicKey, RsaPrivateKey, Vec<u8>);

pub struct RsaKeyPool(usize, VecDeque<RsaKey>);

impl RsaKeyPool {
    pub fn new(key_pool_size: usize) -> Self {
        let vec = VecDeque::with_capacity(key_pool_size);
        Self(key_pool_size, vec)
    }

    pub fn pop(&mut self) -> RsaKey {
        match self.1.pop_front() {
            Some(r) => r,
            None => {
                // FIXME: how??
                panic!("NOT ENOUGH RSA KEYS IN POOL! WILL GENERATE ONE KEY!");

                // let key = RsaPrivateKey::new(&mut rand::thread_rng(), 1024).unwrap();
                // return (key.to_public_key(), key);
            }
        }
    }

    pub fn replenish(this: Arc<Mutex<Self>>, len: Option<usize>) {
        let len = if let Some(len) = len {
            len
        } else {
            this.lock().unwrap().0
        };
        let mut vec = VecDeque::with_capacity(len);
        let mut thread_rng = rand::thread_rng();

        // This is on purpose.
        for _ in 0..len {
            let key = RsaPrivateKey::new(&mut thread_rng, 1024).unwrap();

            // let mut kv = Vec::new();
            let kv = SubjectPublicKeyInfo::from_key(key.to_public_key())
                .unwrap()
                .to_der().unwrap();

            vec.push_back((key.to_public_key(), key, kv));
        }

        this.lock().unwrap().1.extend(vec);
    }

    pub fn fullness(&self) -> f32 {
        self.1.len() as f32 / self.0 as f32
    }
}

pub trait DecryptRsa {
    fn decrypt_ct(&self, data: &[u8]) -> Vec<u8>;
}

impl DecryptRsa for RsaPrivateKey {
    fn decrypt_ct(&self, data: &[u8]) -> Vec<u8> {
        self.decrypt(Pkcs1v15Encrypt, data).unwrap()
    }
}

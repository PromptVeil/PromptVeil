use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use std::io;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
}

impl EncryptedData {
    pub fn new(data: &[u8], key: &[u8; 32]) -> io::Result<Self> {
        let cipher = Aes256Gcm::new(key.into());
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);
        
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce), data)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Encryption failed"))?;

        Ok(Self { ciphertext, nonce })
    }

    pub fn decrypt(&self, key: &[u8; 32]) -> io::Result<Vec<u8>> {
        let cipher = Aes256Gcm::new(key.into());
        
        cipher
            .decrypt(Nonce::from_slice(&self.nonce), self.ciphertext.as_ref())
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Decryption failed"))
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).unwrap()
    }
}

pub fn generate_key() -> io::Result<[u8; 32]> {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    Ok(key)
}

pub fn encrypt(data: &[u8], key: &[u8]) -> io::Result<Vec<u8>> {
    if key.len() != 32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Key must be 32 bytes",
        ));
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid key length"))?;

    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce);
    let nonce = Nonce::from_slice(&nonce);

    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Encryption failed"))?;

    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(nonce.as_slice());
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

pub fn decrypt(data: &[u8], key: &[u8]) -> io::Result<Vec<u8>> {
    if key.len() != 32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Key must be 32 bytes",
        ));
    }

    if data.len() < 12 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Data too short",
        ));
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid key length"))?;

    let nonce = Nonce::from_slice(&data[..12]);
    let ciphertext = &data[12..];

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Decryption failed"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let data = b"Hello, World!";
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);

        let encrypted = encrypt(data, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();

        assert_eq!(data, &decrypted[..]);
    }
} 
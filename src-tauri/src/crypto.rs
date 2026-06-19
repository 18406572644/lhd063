use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::RngCore;
use std::sync::Mutex;

pub struct CryptoService {
    cipher: Mutex<Option<Aes256Gcm>>,
    key: Mutex<Vec<u8>>,
}

impl CryptoService {
    pub fn new() -> Self {
        Self {
            cipher: Mutex::new(None),
            key: Mutex::new(Vec::new()),
        }
    }

    pub fn init_with_key(&self, key_bytes: &[u8]) -> Result<(), String> {
        if key_bytes.len() != 32 {
            return Err("Key must be 32 bytes long".to_string());
        }

        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);

        *self.cipher.lock().unwrap() = Some(cipher);
        *self.key.lock().unwrap() = key_bytes.to_vec();

        Ok(())
    }

    pub fn generate_key() -> Vec<u8> {
        let mut key = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        key
    }

    pub fn get_key_base64(&self) -> String {
        let key = self.key.lock().unwrap();
        BASE64.encode(&*key)
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<String, String> {
        let cipher_guard = self.cipher.lock().unwrap();
        let cipher = cipher_guard
            .as_ref()
            .ok_or_else(|| "Crypto service not initialized".to_string())?;

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(BASE64.encode(&result))
    }

    pub fn decrypt(&self, ciphertext_base64: &str) -> Result<Vec<u8>, String> {
        let cipher_guard = self.cipher.lock().unwrap();
        let cipher = cipher_guard
            .as_ref()
            .ok_or_else(|| "Crypto service not initialized".to_string())?;

        let data = BASE64
            .decode(ciphertext_base64)
            .map_err(|e| format!("Base64 decode failed: {}", e))?;

        if data.len() < 12 {
            return Err("Invalid ciphertext: too short".to_string());
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        Ok(plaintext)
    }

    pub fn encrypt_string(&self, plaintext: &str) -> Result<String, String> {
        self.encrypt(plaintext.as_bytes())
    }

    pub fn decrypt_string(&self, ciphertext_base64: &str) -> Result<String, String> {
        let bytes = self.decrypt(ciphertext_base64)?;
        String::from_utf8(bytes).map_err(|e| format!("UTF-8 decode failed: {}", e))
    }

    pub fn reencrypt_data(
        &self,
        old_key: &[u8],
        new_key: &[u8],
        encrypted_data: &str,
    ) -> Result<String, String> {
        let old_cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(old_key));
        let new_cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(new_key));

        let data = BASE64
            .decode(encrypted_data)
            .map_err(|e| format!("Base64 decode failed: {}", e))?;

        if data.len() < 12 {
            return Err("Invalid ciphertext: too short".to_string());
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = old_cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption with old key failed: {}", e))?;

        let mut new_nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut new_nonce_bytes);
        let new_nonce = Nonce::from_slice(&new_nonce_bytes);

        let new_ciphertext = new_cipher
            .encrypt(new_nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption with new key failed: {}", e))?;

        let mut result = Vec::with_capacity(new_nonce_bytes.len() + new_ciphertext.len());
        result.extend_from_slice(&new_nonce_bytes);
        result.extend_from_slice(&new_ciphertext);

        Ok(BASE64.encode(&result))
    }
}

impl Default for CryptoService {
    fn default() -> Self {
        Self::new()
    }
}

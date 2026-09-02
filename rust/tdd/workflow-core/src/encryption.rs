fn pending<T>() -> T {
    panic!("TDD RED: packages/core/src/encryption.test.ts implementation pending")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyUsage {
    Encrypt,
    Decrypt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CryptoKey {
    pub raw: Vec<u8>,
    pub usages: Vec<KeyUsage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CryptoErrorContext {
    pub operation: String,
    pub byte_length: usize,
    pub format_prefix: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCryptoError {
    pub message: String,
    pub cause_name: Option<String>,
    pub context: CryptoErrorContext,
}

pub fn import_key(raw: &[u8], usages: &[KeyUsage]) -> Result<CryptoKey, RuntimeCryptoError> {
    let _ = (raw, usages);
    pending()
}

pub fn encrypt(key: &CryptoKey, plaintext: &[u8]) -> Result<Vec<u8>, RuntimeCryptoError> {
    let _ = (key, plaintext);
    pending()
}

pub fn decrypt(key: &CryptoKey, ciphertext: &[u8]) -> Result<Vec<u8>, RuntimeCryptoError> {
    let _ = (key, ciphertext);
    pending()
}

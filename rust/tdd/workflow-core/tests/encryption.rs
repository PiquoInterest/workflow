use workflow_core_tdd::encryption::{KeyUsage, decrypt, encrypt, import_key};

const RAW_KEY: [u8; 32] = [7; 32];
const OTHER_RAW_KEY: [u8; 32] = [8; 32];
const SHORT_RAW_KEY: [u8; 16] = [7; 16];

fn key(raw: &[u8]) -> workflow_core_tdd::encryption::CryptoKey {
    import_key(raw, &[KeyUsage::Encrypt, KeyUsage::Decrypt]).unwrap()
}

#[test]
fn encrypt_then_decrypt_returns_the_original_plaintext() {
    let key = key(&RAW_KEY);
    let plaintext = b"hello, workflow";
    let ciphertext = encrypt(&key, plaintext).unwrap();
    assert_eq!(ciphertext.len(), plaintext.len() + 12 + 16);
    assert_eq!(decrypt(&key, &ciphertext).unwrap(), plaintext);
}

#[test]
fn import_rejects_keys_that_are_not_exactly_thirty_two_bytes() {
    let error = import_key(&SHORT_RAW_KEY, &[KeyUsage::Decrypt]).unwrap_err();
    assert!(error.message.contains("must be exactly 32 bytes, got 16"));
}

#[test]
fn short_ciphertext_returns_a_typed_decryption_error() {
    let error = decrypt(&key(&RAW_KEY), &[0; 10]).unwrap_err();
    assert!(error.message.contains("Encrypted data too short"));
    assert_eq!(error.context.operation, "decrypt");
    assert_eq!(error.context.byte_length, 10);
}

#[test]
fn tampered_authentication_tag_is_wrapped_with_operation_error_as_cause() {
    let key = key(&RAW_KEY);
    let mut ciphertext = encrypt(&key, b"hello, workflow").unwrap();
    *ciphertext.last_mut().unwrap() ^= 0xff;
    let error = decrypt(&key, &ciphertext).unwrap_err();
    assert_eq!(error.cause_name.as_deref(), Some("OperationError"));
    assert_eq!(error.context.operation, "decrypt");
    assert_eq!(error.context.byte_length, ciphertext.len());
}

#[test]
fn wrong_key_is_wrapped_as_an_authenticated_decryption_failure() {
    let ciphertext = encrypt(&key(&RAW_KEY), b"secret").unwrap();
    let error = decrypt(&key(&OTHER_RAW_KEY), &ciphertext).unwrap_err();
    assert_eq!(error.cause_name.as_deref(), Some("OperationError"));
}

#[test]
fn low_level_decryption_context_does_not_record_nonce_bytes_as_a_format_prefix() {
    let error = decrypt(&key(&RAW_KEY), &[0x41; 28]).unwrap_err();
    assert_eq!(error.context.operation, "decrypt");
    assert_eq!(error.context.byte_length, 28);
    assert_eq!(error.context.format_prefix, None);
}

#[test]
fn encrypt_wraps_underlying_crypto_usage_failures() {
    let decrypt_only = import_key(&RAW_KEY, &[KeyUsage::Decrypt]).unwrap();
    let error = encrypt(&decrypt_only, b"nope").unwrap_err();
    assert_eq!(error.context.operation, "encrypt");
    assert_eq!(error.context.byte_length, 4);
}

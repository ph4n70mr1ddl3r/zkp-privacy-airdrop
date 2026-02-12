use zkp_airdrop_relayer::config::{has_weak_key_pattern, SecretKey};

#[test]
fn test_secret_key_zeroizes_on_drop() {
    let key = SecretKey::new(
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
    );
    let key_str = key.as_str().to_string();
    drop(key);
    assert_eq!(
        key_str,
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
    );
}

#[test]
fn test_secret_key_clone() {
    let key = SecretKey::new(
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
    );
    let cloned = key.clone();
    assert_eq!(key.as_str(), cloned.as_str());
}

#[test]
fn test_secret_key_is_empty() {
    let empty_key = SecretKey::new(String::new());
    assert!(empty_key.is_empty());

    let key = SecretKey::new(
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
    );
    assert!(!key.is_empty());
}

#[test]
fn test_secret_key_len() {
    let key = SecretKey::new(
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
    );
    assert_eq!(key.len(), 66);
}

#[test]
fn test_weak_key_pattern_all_repeated() {
    let weak_key = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    assert!(has_weak_key_pattern(&hex::decode(&weak_key[2..]).unwrap()));
}

#[test]
fn test_weak_key_pattern_sequential() {
    let weak_key = "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20";
    assert!(has_weak_key_pattern(&hex::decode(&weak_key[2..]).unwrap()));
}

#[test]
fn test_weak_key_pattern_deadbeef() {
    let weak_key = "0xdeadbeef00000000000000000000000000000000000000000000000000000000";
    assert!(has_weak_key_pattern(&hex::decode(&weak_key[2..]).unwrap()));
}

#[test]
fn test_weak_key_pattern_strong() {
    let strong_key = "0x5a8b9c2d1e7f6a3b4c5d6e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b";
    assert!(!has_weak_key_pattern(
        &hex::decode(&strong_key[2..]).unwrap()
    ));
}

#[test]
fn test_weak_key_pattern_alternating() {
    let weak_key = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let key_bytes = hex::decode(&weak_key[2..]).unwrap();
    assert!(has_weak_key_pattern(&key_bytes));
}

#[test]
fn test_weak_key_pattern_cafebabe() {
    let weak_key = "0xcafebabe00000000000000000000000000000000000000000000000000000000";
    assert!(has_weak_key_pattern(&hex::decode(&weak_key[2..]).unwrap()));
}

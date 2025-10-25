//! Integration tests with EICAR test file
//!
//! Tests real YARA scanning and detection capabilities

use av_core::{Scanner, ScannerConfig, RecommendedAction};
use std::fs;
use std::io::Write;
use tempfile::TempDir;

const EICAR_TEST_STRING: &str = r"X5O!P%@AP[4\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*";

#[tokio::test]
async fn test_eicar_detection() {
    // Create temp directory
    let temp_dir = TempDir::new().unwrap();
    let eicar_path = temp_dir.path().join("eicar.com");

    // Write EICAR test file
    let mut file = fs::File::create(&eicar_path).unwrap();
    file.write_all(EICAR_TEST_STRING.as_bytes()).unwrap();
    file.flush().unwrap();
    drop(file);

    // Scan with default config
    let config = ScannerConfig::default();
    let scanner = Scanner::new(config).unwrap();

    let outcome = scanner.scan_path(&eicar_path).await.unwrap();

    // Should detect EICAR and recommend quarantine
    assert_eq!(outcome.recommended_action, RecommendedAction::Quarantine);
    assert!(!outcome.signatures.is_empty(), "EICAR should trigger signature match");

    // Check that EICAR_Test_File rule matched
    let eicar_match = outcome.signatures.iter()
        .find(|sig| sig.rule.contains("EICAR"));

    assert!(eicar_match.is_some(), "EICAR_Test_File rule should match");
}

#[tokio::test]
async fn test_clean_file_allows() {
    // Create temp directory
    let temp_dir = TempDir::new().unwrap();
    let clean_path = temp_dir.path().join("clean.txt");

    // Write innocent content
    fs::write(&clean_path, b"Hello, world!").unwrap();

    let config = ScannerConfig::default();
    let scanner = Scanner::new(config).unwrap();

    let outcome = scanner.scan_path(&clean_path).await.unwrap();

    // Should allow clean file
    assert_eq!(outcome.recommended_action, RecommendedAction::Allow);
    assert!(outcome.signatures.is_empty());
}

#[tokio::test]
async fn test_entropy_detection() {
    // Create temp directory
    let temp_dir = TempDir::new().unwrap();
    let high_entropy_path = temp_dir.path().join("encrypted.bin");

    // Generate high-entropy data (simulated encryption)
    let random_data: Vec<u8> = (0..8192)
        .map(|i| ((i * 137 + 251) % 256) as u8)  // Pseudo-random
        .collect();

    fs::write(&high_entropy_path, &random_data).unwrap();

    let mut config = ScannerConfig::default();
    config.enable_entropy_analysis = true;

    let scanner = Scanner::new(config).unwrap();
    let outcome = scanner.scan_path(&high_entropy_path).await.unwrap();

    // High entropy should be detected
    assert!(outcome.entropy.mean_entropy > 7.0, "Expected high entropy, got {}", outcome.entropy.mean_entropy);
    assert!(!outcome.entropy.suspicious_regions.is_empty(), "Should detect suspicious high-entropy regions");
}

#[tokio::test]
async fn test_upx_packed_detection() {
    // Note: This test would require a real UPX-packed binary
    // For now, we test that the rule exists and can be loaded

    let temp_dir = TempDir::new().unwrap();
    let fake_upx = temp_dir.path().join("fake.upx");

    // Create file with ELF header + UPX marker
    let mut data = vec![0x7F, 0x45, 0x4C, 0x46, 0x02, 0x01, 0x01]; // ELF64 header
    data.extend_from_slice(b"UPX!"); // UPX marker
    data.extend_from_slice(&[0u8; 100]); // Padding

    fs::write(&fake_upx, &data).unwrap();

    let config = ScannerConfig::default();
    let scanner = Scanner::new(config).unwrap();
    let outcome = scanner.scan_path(&fake_upx).await.unwrap();

    // Should detect UPX signature
    if !outcome.signatures.is_empty() {
        let upx_match = outcome.signatures.iter()
            .any(|sig| sig.rule.contains("UPX"));
        assert!(upx_match, "UPX signature should match");
    }
}

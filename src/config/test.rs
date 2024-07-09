use super::load_default_config;

/// Test that all required config options are defined in the default config file.
#[test]
fn test_load_default_config() {
    // As long as this doesn't panic, it means all config was loaded successfully
    let _config = load_default_config();
}

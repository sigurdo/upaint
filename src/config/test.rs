use super::{default_config_source, load_config_from_builder, Config};

/// Test that all required config options are defined in the default config file.
#[test]
fn load_default_config() {
    let builder = ::config::Config::builder().add_source(default_config_source());

    // As long as this doesn't panic, it means all config was loaded successfully
    let _config: Config = load_config_from_builder(builder).unwrap();
}

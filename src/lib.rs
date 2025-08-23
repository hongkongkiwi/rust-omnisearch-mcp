pub mod common;
pub mod config;
pub mod providers;
pub mod server;

pub use config::validate_config;
pub use providers::initialize_providers;
pub use server::{register_tools, setup_handlers};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::types::SearchProvider;

    #[test]
    fn test_config_loading() {
        // This test will pass if the config loads without panicking
        let _config = &*config::CONFIG;
        assert!(true);
    }

    #[test]
    fn test_tavily_provider_creation() {
        let provider = providers::search::TavilySearchProvider::new();
        assert_eq!(provider.name(), "tavily");
        assert!(!provider.description().is_empty());
    }
}

use crate::common::types::{EnhancementProvider, ProcessingProvider, SearchProvider};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

// Track available providers by category
pub static AVAILABLE_PROVIDERS: Lazy<AvailableProviders> = Lazy::new(AvailableProviders::new);

pub struct AvailableProviders {
    pub search: RwLock<std::collections::HashSet<String>>,
    pub ai_response: RwLock<std::collections::HashSet<String>>,
    pub processing: RwLock<std::collections::HashSet<String>>,
    pub enhancement: RwLock<std::collections::HashSet<String>>,
}

impl Default for AvailableProviders {
    fn default() -> Self {
        Self::new()
    }
}

impl AvailableProviders {
    pub fn new() -> Self {
        Self {
            search: RwLock::new(std::collections::HashSet::new()),
            ai_response: RwLock::new(std::collections::HashSet::new()),
            processing: RwLock::new(std::collections::HashSet::new()),
            enhancement: RwLock::new(std::collections::HashSet::new()),
        }
    }
}

// Provider registry
pub struct ToolRegistry {
    search_providers: RwLock<HashMap<String, Box<dyn SearchProvider>>>,
    processing_providers: RwLock<HashMap<String, Box<dyn ProcessingProvider>>>,
    enhancement_providers: RwLock<HashMap<String, Box<dyn EnhancementProvider>>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            search_providers: RwLock::new(HashMap::new()),
            processing_providers: RwLock::new(HashMap::new()),
            enhancement_providers: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_search_provider(
        &self,
        provider: Box<dyn SearchProvider>,
        is_ai_response: bool,
    ) {
        let name = provider.name().to_string();
        self.search_providers
            .write()
            .unwrap()
            .insert(name.clone(), provider);
        if is_ai_response {
            AVAILABLE_PROVIDERS
                .ai_response
                .write()
                .unwrap()
                .insert(name);
        } else {
            AVAILABLE_PROVIDERS.search.write().unwrap().insert(name);
        }
    }

    pub fn register_processing_provider(&self, provider: Box<dyn ProcessingProvider>) {
        let name = provider.name().to_string();
        self.processing_providers
            .write()
            .unwrap()
            .insert(name.clone(), provider);
        AVAILABLE_PROVIDERS.processing.write().unwrap().insert(name);
    }

    pub fn register_enhancement_provider(&self, provider: Box<dyn EnhancementProvider>) {
        let name = provider.name().to_string();
        self.enhancement_providers
            .write()
            .unwrap()
            .insert(name.clone(), provider);
        AVAILABLE_PROVIDERS
            .enhancement
            .write()
            .unwrap()
            .insert(name);
    }
}

// Global registry instance
static REGISTRY: Lazy<ToolRegistry> = Lazy::new(ToolRegistry::new);

pub fn register_tools() -> eyre::Result<()> {
    // This would be implemented based on the rust-mcp-sdk API
    Ok(())
}

pub fn register_search_provider(provider: Box<dyn SearchProvider>, is_ai_response: bool) {
    REGISTRY.register_search_provider(provider, is_ai_response);
}

pub fn register_processing_provider(provider: Box<dyn ProcessingProvider>) {
    REGISTRY.register_processing_provider(provider);
}

pub fn register_enhancement_provider(provider: Box<dyn EnhancementProvider>) {
    REGISTRY.register_enhancement_provider(provider);
}

//! Cache facade (stub).

use crate::config::RiptideConfig;
use crate::error::Result;
use crate::runtime::RiptideRuntime;
use std::sync::Arc;

pub struct CacheFacade {
    config: RiptideConfig,
    runtime: Arc<RiptideRuntime>,
}

impl CacheFacade {
    pub(crate) fn new(config: RiptideConfig, runtime: Arc<RiptideRuntime>) -> Self {
        Self { config, runtime }
    }
}

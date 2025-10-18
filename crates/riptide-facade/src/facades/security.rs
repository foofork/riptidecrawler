//! Security facade (stub).

use crate::config::RiptideConfig;
use crate::error::Result;
use crate::runtime::RiptideRuntime;
use std::sync::Arc;

pub struct SecurityFacade {
    config: RiptideConfig,
    runtime: Arc<RiptideRuntime>,
}

impl SecurityFacade {
    pub(crate) fn new(config: RiptideConfig, runtime: Arc<RiptideRuntime>) -> Self {
        Self { config, runtime }
    }
}

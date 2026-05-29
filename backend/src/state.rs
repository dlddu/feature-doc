//! Shared application state handed to every handler.

use std::sync::Arc;

use crate::config::Config;
use crate::crypto::Kek;
use crate::db::Db;
use crate::github::SharedGitHub;
use crate::llm::SharedValidator;

#[derive(Clone)]
pub struct AppState(Arc<Inner>);

pub struct Inner {
    pub db: Db,
    pub kek: Kek,
    pub config: Config,
    pub github: SharedGitHub,
    pub llm: SharedValidator,
}

impl AppState {
    pub fn new(
        db: Db,
        kek: Kek,
        config: Config,
        github: SharedGitHub,
        llm: SharedValidator,
    ) -> Self {
        AppState(Arc::new(Inner {
            db,
            kek,
            config,
            github,
            llm,
        }))
    }
}

impl std::ops::Deref for AppState {
    type Target = Inner;
    fn deref(&self) -> &Inner {
        &self.0
    }
}

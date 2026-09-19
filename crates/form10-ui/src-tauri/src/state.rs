use std::sync::Mutex;

use form10_core::ImportedSource;

#[derive(Default)]
pub struct AppState {
    pub source: Mutex<Option<ImportedSource>>,
}

use std::sync::Arc;

use crate::services::database::Database;

pub struct AppState {
    pub(crate) database: Database,
}

pub type AppStateArc = Arc<AppState>;

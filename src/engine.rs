use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;

use once_cell::sync::Lazy;
use tokio::runtime::Handle;
use tracing::info;

/// Supporting types for [`Context`], and [`Metadata`]
pub mod context;

use self::context::Context;
use crate::default;
use crate::never;
use crate::panic;
use crate::storage::StorageImpl;
use crate::Blip;
use crate::Blob;
use crate::SqliteStorage;
use crate::Storage;

// Hmm.
// Maybe, "Engine" does not exist.
// Incremental is the only type, and it's merged with StorageExt.
// Arc<dyn Incremental> is blessed.

/// A lazy-initialized [`Engine`] instance storing its results in an in-memory
/// SQLite database. Must be run inside of a [`tokio`] runtime.
pub static EPHEMERAL: Lazy<Engine> = Lazy::new(|| {
    info!("Initializing static EPHEMERAL: Lazy<Engine>");
    Engine::new(Arc::new(SqliteStorage::open_in_memory().unwrap()))
});

/// A lazy-initialized [`Engine`] instance storing its results in a SQLite
/// database in the user's home directory.
pub static PERSISTENT: Lazy<Engine> = Lazy::new(|| {
    info!("Initializing static PERSISTENT: Lazy<Engine>");
    let mut path = home::home_dir().unwrap_or_default();
    path.push(format!(".{}", env!("CARGO_CRATE_NAME", "fiction")));
    if !path.exists() {
        std::fs::create_dir_all(&path).unwrap();
    }
    path.push("db");
    Engine::new(Arc::new(
        SqliteStorage::open(path.to_string_lossy().as_ref()).unwrap(),
    ))
});

/// `Engine` is the main entry point for the library, connecting the storage
/// backend with the query engine.
#[derive(Debug)]
pub struct Engine {
    storage: Arc<dyn StorageImpl>,
}

impl Engine {
    /// Creates a new `Engine` with the given storage backend.
    pub fn new(storage: Arc<dyn StorageImpl>) -> Arc<Engine> {
        Arc::new(Self { storage })
    }

    pub fn storage(&self) -> &Arc<dyn StorageImpl> {
        &self.storage
    }

    fn new_context(&self) -> Context {
        Context::new(self.storage.clone())
    }

    /// Executes a query, returning either a new `Response` or a cached one from
    /// the backing storage.
    pub async fn execute<Request: crate::Request>(
        &self,
        request: Request,
    ) -> Result<Request::Response, never> {
        let request_blip = Blip::new(request);

        let context = self.new_context();

        let response = request.execute(&mut context).await?;

        self.storage.insert_response(&request, &response).await?;
        for alias in request.aliases() {
            self.storage.insert_alias(&alias, &request_blip).await?;
        }

        todo!()
    }
}

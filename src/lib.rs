use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

pub(crate) mod macros;

pub(crate) mod file_system;
pub(crate) mod inode;
pub(crate) mod inode_table;

use file_system::{FileDescriptor, FileSystem, GetAttrError};
use inode_table::INodeTable;

pub struct Contexture {
    mount_path: PathBuf,
    cacheing: bool,
    inner_fs: Arc<FileSystem>,
}

impl Contexture {
    pub fn new(mount_path: PathBuf, timeout: Option<Duration>) -> Self {
        let mount_path = mount_path.canonicalize().expect("canon path");

        let underlying_fd =
            FileDescriptor::open(&mount_path, libc::O_PATH).expect("get handle on path");
        let _state = underlying_fd
            .fstatat("", libc::AT_SYMLINK_NOFOLLOW)
            .expect("get details");

        let mut _inode_table = INodeTable::new();

        let cacheing = timeout.is_some();
        let inner_fs = FileSystem::new(timeout);

        Self {
            mount_path,
            cacheing,
            inner_fs: Arc::new(inner_fs),
        }
    }

    pub async fn run(&self) -> Result<(), ContextureFsError> {
        let mut kernel_config = polyfuse::KernelConfig::default();

        kernel_config.mount_option("nonempty");
        kernel_config.mount_option("default_permissions");
        kernel_config.mount_option(&format!("fsname={}", env!("CARGO_PKG_NAME")));
        kernel_config.mount_option("subtype=intdots");

        kernel_config.export_support(true);
        kernel_config.flock_locks(true);
        kernel_config.writeback_cache(self.cacheing);

        let session = AsyncSession::mount(self.mount_path.clone(), kernel_config)
            .await
            .map_err(ContextureFsError::MountFailed)?;

        while let Some(request) = session.next_request().await.expect("req to be present") {
            tokio::task::spawn(handle_request(self.inner_fs.clone(), request));
        }

        Ok(())
    }
}

async fn handle_request(
    fs: Arc<FileSystem>,
    request: polyfuse::Request,
) -> Result<(), RequestHandlingError> {
    let operation = request.operation().expect("operation");

    let span = tracing::debug_span!("handle_request", req_id = request.unique(), op = ?operation);
    let _enter = span.enter();

    match operation {
        polyfuse::Operation::Getattr(op) => fs.get_attr(&request, op).await?,
        //polyfuse::Operation::Lookup(_op) => {
        //    todo!()
        //}
        //polyfuse::Operation::Opendir(_op) => {
        //    todo!()
        //}
        op => {
            tracing::warn!(?op, "not implemented");

            if let Err(err) = request.reply_error(libc::ENOSYS) {
                tracing::error!("failed to reply with an error: {err}");
            }

            return Err(RequestHandlingError::NotImplemented);
        }
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum ContextureFsError {
    #[error("failed to mount filesystem. Are the fuse libraries available?")]
    MountFailed(std::io::Error),
}

pub(crate) struct AsyncSession {
    inner: tokio::io::unix::AsyncFd<polyfuse::Session>,
}

impl AsyncSession {
    pub(crate) async fn mount(
        mount_path: PathBuf,
        config: polyfuse::KernelConfig,
    ) -> std::io::Result<Self> {
        tokio::task::spawn_blocking(move || {
            let session = polyfuse::Session::mount(mount_path, config)?;
            let inner =
                tokio::io::unix::AsyncFd::with_interest(session, tokio::io::Interest::READABLE)?;

            Ok(Self { inner })
        })
        .await
        .expect("tokio task join error")
    }

    pub(crate) async fn next_request(&self) -> std::io::Result<Option<polyfuse::Request>> {
        futures::future::poll_fn(|cx| {
            let mut guard = futures::ready!(self.inner.poll_read_ready(cx))?;

            match self.inner.get_ref().next_request() {
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    guard.clear_ready();
                    futures::task::Poll::Pending
                }
                res => {
                    guard.retain_ready();
                    futures::task::Poll::Ready(res)
                }
            }
        })
        .await
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum RequestHandlingError {
    #[error("requested filesystem operation is not yet implemented")]
    NotImplemented,

    #[error("an error occurred processing an operation: {0}")]
    OperationError(Box<dyn std::error::Error + Send>),
}

impl From<GetAttrError> for RequestHandlingError {
    fn from(value: GetAttrError) -> Self {
        RequestHandlingError::OperationError(Box::new(value))
    }
}

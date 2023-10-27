use std::path::PathBuf;
use std::sync::Arc;

pub struct ContextureFs {
    mount_path: PathBuf,
    inner_fs: Arc<InnerFs>,
}

impl ContextureFs {
    pub fn new(mount_path: PathBuf) -> Self {
        let inner_fs = Arc::new(InnerFs::default());
        Self { mount_path, inner_fs }
    }

    pub async fn run(&self) -> Result<(), ContextureFsError> {
        let kernel_config = polyfuse::KernelConfig::default();
        let session = AsyncSession::mount(self.mount_path.clone(), kernel_config).await.expect("mount to succeed");

        while let Some(req) = session.next_request().await.expect("req to be present") {
            let _inner_fs = self.inner_fs.clone();

            let _: tokio::task::JoinHandle<std::io::Result<()>> = tokio::task::spawn(async move {
                match req.operation().expect("operation") {
                    // First operations to implement: Getattr, Lookup, Opendir
                    //polyfuse::Operation::Readdir(op) => ...
                    op => {
                        tracing::info!(uid = ?req.uid(), gid = ?req.gid(), op = ?op, "received fuse request");
                        req.reply_error(libc::ENOSYS)?
                    },
                }

                Ok(())
            });
        }

        Ok(())
    }
}

struct InnerFs;

impl InnerFs {
    fn new() -> Self {
        Self
    }
}

impl Default for InnerFs {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ContextureFsError {
}

pub(crate) struct AsyncSession {
    inner: tokio::io::unix::AsyncFd<polyfuse::Session>,
}

impl AsyncSession {
    pub(crate) async fn mount(mount_path: PathBuf, config: polyfuse::KernelConfig) -> std::io::Result<Self> {
        tokio::task::spawn_blocking(move || {
            let session = polyfuse::Session::mount(mount_path, config)?;
            let inner = tokio::io::unix::AsyncFd::with_interest(session, tokio::io::Interest::READABLE)?;

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

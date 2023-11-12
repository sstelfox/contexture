pub(crate) mod file_descriptor;

pub(crate) use file_descriptor::FileDescriptor;

use std::time::Duration;

use tokio::sync::Mutex;

use crate::inode_table::INodeTable;

pub(crate) struct FileSystem {
    timeout: Option<Duration>,
    inodes: Mutex<INodeTable>,
}

impl FileSystem {
    pub(crate) async fn get_attr(
        &self,
        request: &polyfuse::Request,
        _operation: polyfuse::op::Getattr<'_>,
    ) -> Result<(), GetAttrError> {
        let _inode_table = self.inodes.lock().await;

        // for now we don't know about anything
        request
            .reply_error(libc::ENOENT)
            .map_err(GetAttrError::ReplyFailed)
    }

    pub(crate) fn new(timeout: Option<Duration>) -> Self {
        let inodes = Mutex::new(INodeTable::new());

        Self {
            timeout,
            inodes,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum GetAttrError {
    #[error("failed to reply to filesystem request: {0}")]
    ReplyFailed(std::io::Error),
}

pub(crate) mod file_descriptor;

pub(crate) use file_descriptor::FileDescriptor;

use std::sync::Mutex;
use std::time::Duration;

use crate::inode_table::INodeTable;

pub(crate) struct FileSystem {
    //timeout: Option<Duration>,
    //inodes: std::sync::Mutex<INodeTable>,
}

impl FileSystem {
    pub(crate) fn get_attr(
        &self,
        request: &polyfuse::Request,
        _operation: polyfuse::op::Getattr<'_>,
    ) -> Result<(), GetAttrError> {
        // for now we don't know about anything
        request
            .reply_error(libc::ENOENT)
            .map_err(GetAttrError::ReplyFailed)
    }

    pub(crate) fn new(_timeout: Option<Duration>) -> Self {
        let _inodes = Mutex::new(INodeTable::new());

        Self {
            //timeout,
            //inodes,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum GetAttrError {
    #[error("failed to reply to filesystem request: {0}")]
    ReplyFailed(std::io::Error),
}

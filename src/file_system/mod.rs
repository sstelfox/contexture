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
        operation: polyfuse::op::Getattr<'_>,
    ) -> Result<(), GetAttrError> {
        let inode_table = self.inodes.lock().await;

        let inode_mutex = match inode_table.get(&operation.ino()) {
            Some(i) => i,
            None => {
                request
                    .reply_error(libc::ENOENT)
                    .map_err(GetAttrError::ReplyFailed)?;

                return Ok(());
            }
        };

        let inode = inode_mutex.lock().await;
        let inner_stat = match inode.fd.fstatat("", libc::AT_SYMLINK_NOFOLLOW) {
            Ok(is) => is,
            Err(err) => {
                let err_num = err.raw_os_error().unwrap_or(libc::EIO);

                request
                    .reply_error(err_num)
                    .map_err(GetAttrError::ReplyFailed)?;

                return Err(GetAttrError::InnerStat(err));
            }
        };

        let mut op_resp = polyfuse::reply::AttrOut::default();
        copy_stat_attr_to_file_attr(op_resp.attr(), &inner_stat);

        if let Some(timeout) = self.timeout {
            op_resp.ttl(timeout);
        }

        request.reply(op_resp)
            .map_err(GetAttrError::ReplyFailed)?;

        Ok(())
    }

    pub(crate) fn new(timeout: Option<Duration>) -> Self {
        let inodes = Mutex::new(INodeTable::new());

        Self {
            timeout,
            inodes,
        }
    }
}

fn copy_stat_attr_to_file_attr(attr: &mut polyfuse::reply::FileAttr, stat_details: &libc::stat) {
    attr.ino(stat_details.st_ino);
    attr.nlink(stat_details.st_nlink as u32);
    attr.size(stat_details.st_size as u64);

    attr.mode(stat_details.st_mode);
    attr.uid(stat_details.st_uid);
    attr.gid(stat_details.st_gid);

    attr.rdev(stat_details.st_rdev as u32);
    attr.blksize(stat_details.st_blksize as u32);
    attr.blocks(stat_details.st_blocks as u64);

    attr.atime(Duration::new(stat_details.st_atime as u64, stat_details.st_atime_nsec as u32));
    attr.mtime(Duration::new(stat_details.st_mtime as u64, stat_details.st_mtime_nsec as u32));
    attr.ctime(Duration::new(stat_details.st_ctime as u64, stat_details.st_ctime_nsec as u32));
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum GetAttrError {
    #[error("unable to query underlying file descriptor: {0}")]
    InnerStat(std::io::Error),

    #[error("failed to reply to filesystem request: {0}")]
    ReplyFailed(std::io::Error),
}

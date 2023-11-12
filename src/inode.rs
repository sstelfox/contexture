use crate::file_system::FileDescriptor;

pub(crate) type Ino = u64;
pub(crate) type SrcId = (u64, libc::dev_t);

pub(crate) struct INode {
    ino: Ino,
    src_id: SrcId,

    pub(crate) fd: FileDescriptor,

    is_symlink: bool,
    ref_count: u64,
}

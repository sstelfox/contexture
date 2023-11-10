use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::inode::{Ino, INode, SrcId};

pub(crate) struct INodeTable {
    inodes: HashMap<Ino, Arc<Mutex<INode>>>,
    src_to_ino: HashMap<SrcId, Ino>,
    next_ino: u64,
}

impl INodeTable {
    pub(crate) fn new() -> Self {
        INodeTable {
            inodes: HashMap::new(),
            src_to_ino: HashMap::new(),
            // The ino starts at one as the first entry is reserved for the root entry
            next_ino: 1,
        }
    }
}

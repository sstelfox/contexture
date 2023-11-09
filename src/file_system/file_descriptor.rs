use std::ffi::{CString, OsStr};
use std::os::unix::prelude::*;

macro_rules! syscall {
    ($name:ident ( $($args:expr),* $(,)? )) => {
        match unsafe { libc::$name($($args),*) } {
            -1 => Err(std::io::Error::last_os_error()),
            ret => Ok(ret),
        }
    }
}

pub(crate) struct FileDescriptor(RawFd);

impl FileDescriptor {
    pub(crate) fn fstatat(
        &self,
        _path: impl AsRef<OsStr>,
        mut _flags: libc::c_int,
    ) -> std::io::Result<libc::stat> {
        todo!()
    }

    pub(crate) fn open(path: impl AsRef<OsStr>, mut flags: libc::c_int) -> std::io::Result<Self> {
        let path = path.as_ref();

        if path.is_empty() {
            flags |= libc::AT_EMPTY_PATH;
        }

        let c_path = CString::new(path.as_bytes())?;
        let fd = syscall!(open(c_path.as_ptr(), flags))?;

        Ok(Self(fd))
    }
}

impl Drop for FileDescriptor {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.0);
        }
    }
}

impl AsRawFd for FileDescriptor {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl FromRawFd for FileDescriptor {
    #[inline]
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self(fd)
    }
}

impl IntoRawFd for FileDescriptor {
    #[inline]
    fn into_raw_fd(self) -> RawFd {
        self.0
    }
}

use bcc_sys::bccapi::*;
use failure::*;

use std::ffi::CString;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::os::unix::prelude::*;

#[derive(Debug)]
pub struct Tracepoint {
    subsys: CString,
    name: CString,
    code_fd: File,
    p: i32,
}

impl Tracepoint {
    pub fn attach_tracepoint(subsys: &str, name: &str, file: File) -> Result<Self, Error> {
        let cname =
            CString::new(name).map_err(|_| format_err!("Nul byte in Tracepoint name: {}", name))?;
        let csubsys = CString::new(subsys)
            .map_err(|_| format_err!("Nul byte in Tracepoint subsys: {}", subsys))?;
        let ptr = unsafe {
            bpf_attach_tracepoint(
                file.as_raw_fd(),
                csubsys.as_ptr(),
                cname.as_ptr(),
            )
        };
        if ptr < 0 {
            return Err(format_err!(
                "Failed to attach tracepoint: {}:{}",
                subsys,
                name
            ));
        } else {
            Ok(Self {
                subsys: csubsys,
                name: cname,
                code_fd: file,
                p: ptr,
            })
        }
    }
}

impl PartialEq for Tracepoint {
    fn eq(&self, other: &Tracepoint) -> bool {
        self.subsys == other.subsys && self.name == other.name
    }
}

impl Eq for Tracepoint {}

impl Hash for Tracepoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.subsys.hash(state);
        self.name.hash(state);
    }
}

impl Drop for Tracepoint {
    fn drop(&mut self) {
        unsafe {
            bpf_detach_tracepoint(self.subsys.as_ptr(), self.name.as_ptr());
        }
    }
}

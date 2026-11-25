//! The gate of the UI link's Windows pipe (#3874): the security
//! descriptor that admits the window's own user only, and the pipe
//! instance creation that carries it. The kernel - not a token on the
//! wire - refuses another local user the pipe open, before our code
//! runs.
//!
//! Provenance: our own FFI over the platform's published security API
//! (Microsoft Learn: ConvertStringSecurityDescriptorToSecurityDescriptorW,
//! GetTokenInformation, CreateNamedPipeW). Clean-room, public sources
//! only.

#![allow(clippy::undocumented_unsafe_blocks)] // every block names its API below

use std::ffi::OsStr;
use std::io;
use std::os::windows::ffi::OsStrExt;

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE, LocalFree};
use windows_sys::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows_sys::Win32::Security::{
    GetTokenInformation, PSECURITY_DESCRIPTOR, PSID, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER,
    TokenUser,
};
use windows_sys::Win32::Storage::FileSystem::{
    FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_FLAG_OVERLAPPED,
};
use windows_sys::Win32::System::Pipes::{
    CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

/// One read or write buffer of a pipe instance.
const PIPE_BUF: u32 = 64 * 1024;

/// The SDDL descriptor string `D:P(A;;GA;;;<user SID>)`: access granted
/// to the current user and to nobody else.
fn user_only_sddl() -> io::Result<String> {
    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut len = 0_u32;
        let _ = GetTokenInformation(token, TokenUser, std::ptr::null_mut(), 0, &mut len);
        let mut buf = vec![0_u8; len.max(4) as usize];
        let said = GetTokenInformation(token, TokenUser, buf.as_mut_ptr().cast(), len, &mut len);
        CloseHandle(token);
        if said == 0 {
            return Err(io::Error::last_os_error());
        }
        let user = buf.as_ptr().cast::<TOKEN_USER>();
        let sid: PSID = (*user).User.Sid;
        let mut text = std::ptr::null_mut();
        if ConvertSidToStringSidW(sid, &mut text) == 0 || text.is_null() {
            return Err(io::Error::last_os_error());
        }
        let mut utf16 = Vec::new();
        let mut i = 0_usize;
        while *text.add(i) != 0 {
            utf16.push(*text.add(i));
            i += 1;
        }
        LocalFree(text.cast());
        let sid = String::from_utf16(&utf16)
            .map_err(|_| io::Error::other("the user SID is not UTF-16"))?;
        Ok(format!("D:P(A;;GA;;;{sid})"))
    }
}

/// A pipe security descriptor that admits the current user only. The
/// kernel copies it into each instance, so it is freed after the last
/// `CreateNamedPipeW` that borrowed it (the listener's own lifetime).
pub(super) struct SecurityDescriptor(SECURITY_ATTRIBUTES);

impl SecurityDescriptor {
    /// The `D:P(A;;GA;;;<user SID>)` descriptor of this process's user.
    ///
    /// # Errors
    /// The token or the descriptor conversion failed.
    pub(super) fn user_only() -> io::Result<Self> {
        let wide = OsStr::new(&user_only_sddl()?)
            .encode_wide()
            .chain([0])
            .collect::<Vec<_>>();
        let mut sd: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
        let made = unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                wide.as_ptr(),
                SDDL_REVISION_1,
                &mut sd,
                std::ptr::null_mut(),
            )
        };
        if made == 0 || sd.is_null() {
            return Err(io::Error::last_os_error());
        }
        let attributes = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: sd.cast(),
            bInheritHandle: 0,
        };
        Ok(Self(attributes))
    }

    /// The raw attributes each `CreateNamedPipeW` call borrows.
    pub(super) fn attributes(&self) -> &SECURITY_ATTRIBUTES {
        &self.0
    }
}

impl Drop for SecurityDescriptor {
    fn drop(&mut self) {
        unsafe { LocalFree(self.0.lpSecurityDescriptor.cast()) };
    }
}

/// One pipe instance waiting for a client. `first` adds the
/// anti-hijack flag: the name must be free, or the window's link fails
/// to open (a squatter cannot shadow a real window).
///
/// # Errors
/// The pipe creation failed (the name is taken, or the OS refused).
pub(super) fn create_instance(
    name: &[u16],
    sd: &SECURITY_ATTRIBUTES,
    first: bool,
) -> io::Result<HANDLE> {
    let mode = windows_sys::Win32::Storage::FileSystem::PIPE_ACCESS_DUPLEX
        | FILE_FLAG_OVERLAPPED
        | if first {
            FILE_FLAG_FIRST_PIPE_INSTANCE
        } else {
            0
        };
    let kind = PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT;
    // FILE_FLAG_OVERLAPPED: every wait on this handle carries its own
    // deadline (the synchronous pipe API has none).
    let handle = unsafe {
        CreateNamedPipeW(
            name.as_ptr(),
            mode,
            kind,
            PIPE_UNLIMITED_INSTANCES,
            PIPE_BUF,
            PIPE_BUF,
            0,
            sd,
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        Err(io::Error::last_os_error())
    } else {
        Ok(handle)
    }
}

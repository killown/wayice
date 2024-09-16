use crate::core::state::{Backend, WayiceState};
use crate::helpers::window_utils::{get_window_info, get_x11_window_info};
use libc::sem_t;
use libc::{
    close, ftruncate, mmap, munmap, sem_close, sem_open, sem_post, sem_wait, shm_open, MAP_FAILED,
    MAP_SHARED, O_CREAT, O_RDWR, PROT_READ, PROT_WRITE,
};
use serde_json::json;
use std::ffi::CString;
use std::ptr;

/// Sets a string value in a shared memory object.
///
/// This function creates or opens a shared memory object with the specified name and writes the given string
/// value to it. If the shared memory object already exists, it will be cleared and resized to accommodate the new string.
///
/// # Parameters
///
/// - `shm_name`: A string slice (`&str`) representing the name of the shared memory object. This name is used to
///   create or open the shared memory segment. It should be unique to avoid conflicts with other shared memory
///   objects.
/// - `value`: A string slice (`&str`) representing the value to be written to the shared memory. This string will be
///   copied into the shared memory segment and null-terminated to ensure it can be read as a proper C-style string.
///
/// # Behavior
///
/// 1. **Open or Create Shared Memory**: The function attempts to open or create a shared memory object with the
///    specified name (`shm_name`). It uses `libc::shm_open` with flags for creation (`O_CREAT`) and read/write access (`O_RDWR`).
///    If the operation fails, an error message is printed and the function returns without making further changes.
///
/// 2. **Truncate Shared Memory**: The function truncates the shared memory object to zero size to clear any existing data.
///    This is done using `libc::ftruncate` with a size of zero. If this operation fails, an error message is printed, the
///    shared memory object is closed, and the function returns.
///
/// 3. **Set Size of Shared Memory**: The function then sets the size of the shared memory object to the length of the
///    string plus one byte for the null terminator. This is done using `libc::ftruncate` with the calculated size. If this
///    operation fails, an error message is printed, the shared memory object is closed, and the function returns.
///
/// 4. **Map Shared Memory**: The function maps the shared memory object into the process's address space using `libc::mmap`.
///    It requests read and write access to the memory. If mapping fails, an error message is printed, the shared memory
///    object is closed, and the function returns.
///
/// 5. **Write to Shared Memory**: The function copies the string value into the mapped memory and appends a null terminator
///    to ensure the string is properly null-terminated. This is done using `ptr::copy_nonoverlapping` and direct pointer
///    manipulation.
///
/// 6. **Clean Up**: Finally, the function unmaps the shared memory using `libc::munmap` and closes the file descriptor
///    using `libc::close`. This ensures that the resources are properly released.
///
/// # Errors
///
/// The function may print error messages and return without making changes if any of the following operations fail:
/// - Opening or creating the shared memory object.
/// - Truncating the shared memory object to zero size.
/// - Setting the size of the shared memory object.
/// - Mapping the shared memory object into memory.
///
/// # Safety
///
/// This function uses unsafe Rust to perform low-level operations such as memory mapping and direct pointer manipulation.
/// The caller should ensure that the string value does not contain null bytes other than the final terminator, as this
/// may cause unexpected behavior.
///
/// # Example
///
/// ```rust
/// let shm_name = "/wayice_shared_memory_variable";
/// let value = "Hello, shared memory!";
/// ipc_set_string(shm_name, value);
/// ```

fn generate_sem_name(shm_name: &str) -> CString {
    let sem_name = format!("/semaphore_{}", shm_name.trim_start_matches('/'));
    CString::new(sem_name).unwrap()
}

pub fn ipc_set_string(shm_name: &str, value: &str) {
    // Generate semaphore name based on shared memory name
    let sem_name = generate_sem_name(shm_name);

    // Create or open semaphore
    let sem_fd: *mut sem_t = unsafe { sem_open(sem_name.as_ptr(), libc::O_CREAT | libc::O_RDWR, 0o666, 1) };
    if sem_fd.is_null() {
        eprintln!("Failed to open or create semaphore");
        return;
    }

    // Acquire semaphore
    unsafe {
        if sem_wait(sem_fd) == -1 {
            eprintln!("Failed to acquire semaphore");
            sem_close(sem_fd);
            return;
        }
    }

    // Create or open shared memory object
    let shm_name = CString::new(shm_name).unwrap();
    let shm_fd = unsafe { shm_open(shm_name.as_ptr(), libc::O_CREAT | libc::O_RDWR, 0o666) };
    if shm_fd == -1 {
        eprintln!("Failed to open shared memory");
        unsafe { sem_post(sem_fd) }; // Release semaphore
        unsafe { sem_close(sem_fd) };
        return;
    }

    // Set size of shared memory
    let size = value.len() + 1; // Include space for null terminator
    if unsafe { ftruncate(shm_fd, size as libc::off_t) } == -1 {
        eprintln!("Failed to set size of shared memory");
        unsafe { close(shm_fd) };
        unsafe { sem_post(sem_fd) }; // Release semaphore
        unsafe { sem_close(sem_fd) };
        return;
    }

    // Map shared memory
    let mapped_mem = unsafe {
        mmap(
            ptr::null_mut(),
            size,
            PROT_READ | PROT_WRITE,
            MAP_SHARED,
            shm_fd,
            0,
        )
    };
    if mapped_mem == MAP_FAILED {
        eprintln!("Failed to map shared memory");
        unsafe { close(shm_fd) };
        unsafe { sem_post(sem_fd) }; // Release semaphore
        unsafe { sem_close(sem_fd) };
        return;
    }

    // Write to shared memory
    unsafe {
        let mem_ptr = mapped_mem as *mut u8;
        ptr::copy_nonoverlapping(value.as_ptr(), mem_ptr, value.len());
        *(mem_ptr.add(value.len())) = 0; // Null-terminate
    }

    // Clean up
    unsafe { munmap(mapped_mem, size) };
    unsafe { close(shm_fd) };
    unsafe { sem_post(sem_fd) }; // Release semaphore
    unsafe { sem_close(sem_fd) }; // Close semaphore
}

impl<BackendData: Backend> WayiceState<BackendData> {
    pub fn ipc_shm_update_window_list(&mut self) {
        let surfaces: Vec<String> = self
            .space
            .elements()
            .filter_map(|window| {
                window.wl_surface().map(|surface| {
                    if window.is_x11() {
                        if let Some(x11_surface) = window.0.x11_surface() {
                            get_x11_window_info(x11_surface)
                        } else {
                            json!({"error": "Invalid X11 surface"}).to_string()
                        }
                    } else {
                        get_window_info(surface.as_ref())
                    }
                })
            })
            .collect();

        let result_str = format!("[{}]", surfaces.join(","));

        ipc_set_string("/wayice_list_windows", &result_str);
    }
}

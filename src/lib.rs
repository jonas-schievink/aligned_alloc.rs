extern crate libc;
extern crate winapi;
extern crate kernel32;

/// Allocates `size` Bytes aligned to `align` Bytes. Returns a null pointer on allocation failure.
///
/// The returned pointer must be deallocated by using `aligned_free`.
///
/// # Parameters
///
/// * `size`: The size of the allocation in bytes.
/// * `align`: The alignment of the allocation (at least the size of `usize` on the current
///   platform). Must also be a power of two.
#[inline]
pub fn aligned_alloc(size: usize, align: usize) -> *mut () {
    imp::aligned_alloc(size, align)
}

/// Deallocates aligned memory that was allocated with `aligned_alloc`. Unsafe because calling this
/// with a pointer that was not allocated with `aligned_alloc` (or already released) causes
/// undefined behavior.
#[inline]
pub unsafe fn aligned_free(ptr: *mut ()) {
    imp::aligned_free(ptr)
}

#[cfg(unix)]
mod imp {
    use libc::{c_void, c_int, size_t, EINVAL, ENOMEM, free};

    use std::{mem, ptr};

    extern "C" {
        fn posix_memalign(memptr: *mut *mut c_void, alignment: size_t, size: size_t) -> c_int;
    }

    pub fn aligned_alloc(size: usize, align: usize) -> *mut () {
        let mut memptr: *mut c_void = ptr::null_mut();
        let result = unsafe { posix_memalign(&mut memptr, align as size_t, size as size_t) };
        match result {
            0 => return memptr as *mut (),
            EINVAL => {
                if align < mem::size_of::<usize>() {
                    panic!("EINVAL: invalid alignment: {} (minimum is {})", align,
                        mem::size_of::<usize>());
                }
                if !align.is_power_of_two() {
                    panic!("EINVAL: invalid alignment: {} (must be a power of two)", align)
                }
                panic!("EINVAL: invalid alignment: {}", align);
            }
            ENOMEM => {
                // FIXME: Maybe handle this better
                panic!("ENOMEM: ran out of memory when attempting aligned allocation of {} bytes \
                    (aligned to {} bytes)", size, align)
            }
            _ => unreachable!(),
        }
    }

    pub unsafe fn aligned_free(ptr: *mut ()) {
        free(ptr as *mut c_void)
    }
}

#[cfg(windows)]
mod imp {
    use kernel32::VirtualAlloc;

    use std::{mem, ptr};

    pub fn aligned_alloc(size: usize, align: usize) -> *mut () {
        panic!()
    }

    pub unsafe fn aligned_free(ptr: *mut ()) {
        panic!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let ptr = aligned_alloc(1, 1024 * 1024);
        assert!(!ptr.is_null());
        assert!(ptr as usize % (1024 * 1024) == 0);
        unsafe { aligned_free(ptr) }
    }
}

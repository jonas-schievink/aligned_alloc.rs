#![cfg(any(unix, windows))]

/// Allocates `size` Bytes aligned to `align` Bytes. Returns a null pointer on allocation failure.
///
/// The returned pointer must be deallocated by using `aligned_free`.
///
/// Note: This function is meant to be used for infrequent large allocations (as `malloc` already
/// guarantees suitable alignment for all native datatypes) and might be quite slow when used
/// heavily.
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
    extern crate libc;

    use self::libc::{c_void, c_int, size_t, EINVAL, ENOMEM, free};

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
            ENOMEM => return ptr::null_mut(),
            _ => unreachable!(),
        }
    }

    #[inline]
    pub unsafe fn aligned_free(ptr: *mut ()) {
        free(ptr as *mut c_void)
    }
}

#[cfg(windows)]
mod imp {
    extern crate winapi;
    extern crate kernel32;

    use self::kernel32::{GetLastError, GetSystemInfo, VirtualAlloc, VirtualFree};
    use self::winapi::{MEM_COMMIT, MEM_RESERVE, MEM_RELEASE, PAGE_NOACCESS, PAGE_READWRITE, SIZE_T,
        LPVOID, DWORD, SYSTEM_INFO};

    use std::mem;
    use std::ptr;

    static mut PAGE_SIZE: DWORD = 0;

    #[cold]
    fn get_page_size() {
        let mut info: SYSTEM_INFO = unsafe { mem::uninitialized() };
        unsafe { GetSystemInfo(&mut info); }

        unsafe {
            PAGE_SIZE = info.dwPageSize;
        }
    }

    pub fn aligned_alloc(size: usize, align: usize) -> *mut () {
        assert!(align.is_power_of_two(), "align must be a power of two");

        if unsafe { PAGE_SIZE } == 0 { get_page_size() }

        unsafe {
            if align <= PAGE_SIZE as usize {
                // Page alignment is guaranteed by `VirtualAlloc`
                let ptr = VirtualAlloc(ptr::null_mut(), size as SIZE_T, MEM_COMMIT | MEM_RESERVE,
                    PAGE_READWRITE);
                return ptr as *mut ()
            }

            // Step 1: Reserve `size+align-1` Bytes of address space to find a suitable address
            let ptr = VirtualAlloc(ptr::null_mut(), (size + align - 1) as SIZE_T, MEM_RESERVE,
                PAGE_NOACCESS);
            if ptr.is_null() { return ptr::null_mut() }

            // Step 2: Calculate an aligned address within the reserved range
            // (this works because `align` must be a power of two)
            let aligned_ptr = (ptr as usize + align - 1) & !(align - 1);

            // Step 3: Actually allocate (commit) the memory
            let res = VirtualFree(ptr as LPVOID, 0, MEM_RELEASE);
            if res == 0 {
                panic!("WINAPI error {} while freeing reserved memory", GetLastError());
            }
            let ptr = VirtualAlloc(aligned_ptr as LPVOID, size as SIZE_T, MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE);
            ptr as *mut ()
        }
    }

    pub unsafe fn aligned_free(ptr: *mut ()) {
        let res = VirtualFree(ptr as LPVOID, 0, MEM_RELEASE);
        if res == 0 {
            panic!("WINAPI error {} while releasing memory", GetLastError());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_low_align() {
        let ptr = aligned_alloc(1, 128);
        assert!(!ptr.is_null());
        assert!(ptr as usize % 128 == 0);
        unsafe { aligned_free(ptr) }
    }

    #[test]
    fn small_high_align() {
        let ptr = aligned_alloc(1, 1024 * 1024);
        assert!(!ptr.is_null());
        assert!(ptr as usize % (1024 * 1024) == 0);
        unsafe { aligned_free(ptr) }
    }

    #[test]
    fn large_high_align() {
        // allocate 1 MiB aligned to 1 MiB
        let ptr = aligned_alloc(1024 * 1024, 1024 * 1024);
        assert!(!ptr.is_null());
        assert!(ptr as usize % (1024 * 1024) == 0);
        unsafe { aligned_free(ptr) }
    }

    #[test]
    #[should_panic]
    fn align_less_than_sizeof_usize() {
        // alignment of less than sizeof(usize)
        aligned_alloc(1, 3);
    }

    #[test]
    #[should_panic]
    fn align_not_power_of_two() {
        aligned_alloc(1, 27);
    }

    #[test]
    #[should_panic]
    fn align_zero() {
        aligned_alloc(1, 0);
    }
}

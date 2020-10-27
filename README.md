# Deprecated

**This crate is deprecated in favor of the built-in allocation APIs in `std::alloc`.**

# Aligned Allocations for Rust [![](http://meritbadge.herokuapp.com/aligned_alloc)](https://crates.io/crates/aligned_alloc)

| Linux / OS X | Windows |
| :----------: | :-----: |
| [![Build Status](https://travis-ci.org/jonas-schievink/aligned_alloc.rs.svg?branch=master)](https://travis-ci.org/jonas-schievink/aligned_alloc.rs) | [![Build status](https://ci.appveyor.com/api/projects/status/87oi2nolh91715px/branch/master?svg=true)](https://ci.appveyor.com/project/jonas-schievink/aligned-alloc-rs/branch/master) |

This crate provides cross-platform primitives for requesting specifically
aligned allocations. It is **not** meant to be used as a general allocator API
like `alloc::heap`, but can be used for infrequent large allocations that have a
specific alignment requirement.

For example, certain arena allocators can find the arena in which an object was
allocated by masking the address bits if the arena is aligned to its size.

On Unix, this crate makes use of the `posix_memalign` function. On Windows, it's
a little more complicated: We use `VirtualAlloc` to reserve a chunk of address
space large enough for the allocation plus alignment (no memory is actually
allocated), then calculate an aligned address inside this reserved space, undo
the reservation with `VirtualFree` (the extra bit of reserved memory for the
alignment won't get wasted), and `VirtualAlloc` again, this time passing the aligned pointer and
actually allocating the memory instead of just reserving address space.

# Usage

As usual, in your `Cargo.toml`:
```toml
[dependencies]
aligned_alloc = "0.1"
```

And in your `lib.rs` or `bin.rs`:
```rust
extern crate aligned_alloc;
```

# API

The API is simple, there are just two methods:

#### `fn aligned_alloc(size: usize, align: usize) -> *mut ()`

Allocates `size` Bytes aligned to `align` Bytes. Returns a null pointer on allocation failure.

The returned pointer must be deallocated by using `aligned_free`.

**Note**: This function is meant to be used for infrequent large allocations (as `malloc` already
guarantees suitable alignment for all native datatypes) and might be quite slow when used
heavily. Consider using [`Vec::with_capacity`] combined with [`Vec::as_ptr`] and perhaps
[`mem::forget`] and [`Vec::from_raw_parts`] if you need generic allocation.

##### Parameters

 * `size`: The size of the allocation in bytes.
 * `align`: The alignment of the allocation (at least the size of `usize` on the current
   platform). Must also be a power of two.

#### `unsafe fn aligned_free(ptr: *mut ())`

Deallocates aligned memory that was allocated with `aligned_alloc`. Unsafe because calling this
with a pointer that was not allocated with `aligned_alloc` (or already released) causes
undefined behavior.

[`Vec::with_capacity`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.with_capacity
[`Vec::as_ptr`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.as_ptr
[`mem::forget`]: https://doc.rust-lang.org/std/mem/fn.forget.html
[`Vec::from_raw_parts`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.from_raw_parts

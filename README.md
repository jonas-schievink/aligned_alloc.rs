# Aligned Allocations for Rust

| Linux / OS X Build |
| ------------------ |
| [![Build Status](https://travis-ci.org/jonas-schievink/aligned_alloc.rs.svg?branch=master)](https://travis-ci.org/jonas-schievink/aligned_alloc.rs) |

This crate provides cross-platform primitives for requesting specifically
aligned allocations. It is **not** meant to be used as a general allocator API
like `alloc::heap`, but can be used for infrequent large allocations that have a
specific alignment requirement.

For example, certain arena allocators can find the arena in which an object was
allocated by masking the address bits if the arena is aligned to its size.

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
heavily.

##### Parameters

 * `size`: The size of the allocation in bytes.
 * `align`: The alignment of the allocation (at least the size of `usize` on the current
   platform). Must also be a power of two.

#### `unsafe fn aligned_free(ptr: *mut ())`

Deallocates aligned memory that was allocated with `aligned_alloc`. Unsafe because calling this
with a pointer that was not allocated with `aligned_alloc` (or already released) causes
undefined behavior.

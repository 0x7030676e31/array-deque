# ArrayDeque

[![Crates.io](https://img.shields.io/crates/v/array-deque.svg)](https://crates.io/crates/array-deque)
[![Documentation](https://docs.rs/array-deque/badge.svg)](https://docs.rs/array-deque)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A collection of fixed-capacity circular buffer (ring buffer) implementations providing efficient double-ended queue operations. This crate offers both heap-allocated and stack-allocated variants, making it suitable for a wide range of use cases from general applications to embedded systems.

## Implementations

This crate provides two deque implementations:

- **`ArrayDeque`**: Heap-allocated with runtime-determined capacity
- **`StackArrayDeque`**: Stack-allocated with compile-time fixed capacity

Both implementations use circular buffers for efficient O(1) operations at both ends and will overwrite old elements when full.

## Features

- **Fixed Capacity**: Memory usage is determined at creation/compile time
- **Circular Buffer**: Efficient O(1) operations at both ends
- **Overwrite Behavior**: When full, new elements overwrite the oldest ones
- **Zero Allocations**: After initial allocation, no further memory allocations
- **Stack Allocation**: `StackArrayDeque` uses no heap memory at all
- **No-std Support**: Works in `no_std` environments (with `alloc` for `ArrayDeque`)
- **Serde Support**: Optional serialization/deserialization (with `serde` feature)
- **Iterator Support**: Full iterator implementation with `IntoIterator`
- **Index Access**: Direct element access via indexing
- **Clone Support**: Deep cloning of the entire deque

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
array-deque = "0.3.1"

# For serde support
array-deque = { version = "0.3.1", features = ["serde"] }

# For no_std environments (requires alloc for ArrayDeque)
array-deque = { version = "0.3.1", default-features = false }

# For no_std with serde
array-deque = { version = "0.3.1", default-features = false, features = ["serde"] }
```

## Usage

### Choosing Between ArrayDeque and StackArrayDeque

**Use `ArrayDeque` when:**

- Capacity is determined at runtime
- You need flexibility in sizing
- Memory usage is not severely constrained
- Working in environments with heap allocation

**Use `StackArrayDeque` when:**

- Capacity is known at compile time
- You want zero heap allocations
- Working in embedded or no-heap environments
- Maximum performance and predictability is required

### ArrayDeque (Heap-allocated)

```rust
use array_deque::ArrayDeque;

// Create a deque with capacity for 5 elements
let mut deque = ArrayDeque::new(5);

// Add elements to the back
deque.push_back(1);
deque.push_back(2);
deque.push_back(3);

// Add elements to the front
deque.push_front(0);

// Access elements by index (0 = front, len-1 = back)
assert_eq!(deque[0], 0);  // front element
assert_eq!(deque[1], 1);
assert_eq!(deque[2], 2);
assert_eq!(deque[3], 3);  // back element

// Remove elements
assert_eq!(deque.pop_front(), Some(0));
assert_eq!(deque.pop_back(), Some(3));
```

### StackArrayDeque (Stack-allocated)

```rust
use array_deque::StackArrayDeque;

// Create a stack-allocated deque with compile-time capacity
let mut deque: StackArrayDeque<i32, 5> = StackArrayDeque::new();

// Same API as ArrayDeque
deque.push_back(1);
deque.push_back(2);
deque.push_back(3);
deque.push_front(0);

assert_eq!(deque[0], 0);
assert_eq!(deque[1], 1);
assert_eq!(deque[2], 2);
assert_eq!(deque[3], 3);

assert_eq!(deque.pop_front(), Some(0));
assert_eq!(deque.pop_back(), Some(3));
```

### Overflow Behavior

Both implementations behave identically when reaching capacity:

```rust
use array_deque::{ArrayDeque, StackArrayDeque};

// Heap-allocated version
let mut heap_deque = ArrayDeque::new(3);
heap_deque.push_back(1);
heap_deque.push_back(2);
heap_deque.push_back(3);
heap_deque.push_back(4);  // Overwrites 1
// Deque is now: [2, 3, 4]

// Stack-allocated version
let mut stack_deque: StackArrayDeque<i32, 3> = StackArrayDeque::new();
stack_deque.push_back(1);
stack_deque.push_back(2);
stack_deque.push_back(3);
stack_deque.push_back(4);  // Overwrites 1
// Deque is now: [2, 3, 4]
```

### Iteration

Both types support the same iteration patterns:

```rust
use array_deque::{ArrayDeque, StackArrayDeque};

// ArrayDeque
let mut heap_deque = ArrayDeque::new(5);
heap_deque.extend(vec![1, 2, 3, 4, 5]);

// StackArrayDeque
let mut stack_deque: StackArrayDeque<i32, 5> = StackArrayDeque::new();
for i in 1..=5 {
    stack_deque.push_back(i);
}

// Both support iteration by reference
for item in &heap_deque {
    println!("{}", item);
}

for item in stack_deque.iter() {
    println!("{}", item);
}
```

### Creating from Collections

`ArrayDeque` supports creation from various collections:

```rust
use array_deque::ArrayDeque;

// From array
let deque = ArrayDeque::from([1, 2, 3, 4, 5]);

// From vector
let deque = ArrayDeque::from(vec![1, 2, 3]);

// From slice
let slice = &[1, 2, 3];
let deque = ArrayDeque::from(slice);

// From iterator
let deque: ArrayDeque<i32> = (1..=5).collect();
```

`StackArrayDeque` must be created with `new()` and populated manually:

```rust
use array_deque::StackArrayDeque;

let mut deque: StackArrayDeque<i32, 5> = StackArrayDeque::new();
for i in 1..=5 {
    deque.push_back(i);
}
```

### No-std Usage

Both types work in `no_std` environments:

```rust
#![no_std]
extern crate alloc;  // Only needed for ArrayDeque

use array_deque::{ArrayDeque, StackArrayDeque};
use alloc::vec;

// ArrayDeque requires alloc
let mut heap_deque = ArrayDeque::new(3);
heap_deque.extend(vec![1, 2, 3]);

// StackArrayDeque works without alloc
let mut stack_deque: StackArrayDeque<i32, 3> = StackArrayDeque::new();
stack_deque.push_back(1);
stack_deque.push_back(2);
stack_deque.push_back(3);
```

### Serde Support

With the `serde` feature enabled, both types support serialization:

```rust
use array_deque::{ArrayDeque, StackArrayDeque};
use serde_json;

// ArrayDeque
let mut heap_deque = ArrayDeque::new(3);
heap_deque.extend(vec![1, 2, 3]);
let json = serde_json::to_string(&heap_deque).unwrap();

// StackArrayDeque
let mut stack_deque: StackArrayDeque<i32, 3> = StackArrayDeque::new();
stack_deque.push_back(1);
stack_deque.push_back(2);
stack_deque.push_back(3);
let json_stack = serde_json::to_string(&stack_deque).unwrap();
```

## API Overview

Both `ArrayDeque` and `StackArrayDeque` share the same core API:

### Core Methods

- `new()` - Create a new deque (`new(capacity)` for `ArrayDeque`, `new()` for `StackArrayDeque`)
- `push_back(item)` - Add element to the back
- `push_front(item)` - Add element to the front
- `pop_back()` - Remove and return back element
- `pop_front()` - Remove and return front element

### Capacity and State

- `len()` - Number of elements currently stored
- `capacity()` - Maximum number of elements
- `is_empty()` - Check if deque has no elements
- `is_full()` - Check if deque is at capacity
- `clear()` - Remove all elements

### Access and Iteration

- `[index]` - Direct element access and mutation
- `iter()` - Iterator over element references

### ArrayDeque Additional Methods

- `into_iter()` - Consuming iterator
- `extend()` - Extend from iterator
- Various `From` implementations

## Performance

All core operations are O(1) for both implementations:

- `push_back` / `push_front`: O(1)
- `pop_back` / `pop_front`: O(1)
- Index access: O(1)
- `len` / `is_empty` / `is_full`: O(1)

## Use Cases

### Heap-Allocated ArrayDeque

- **Audio/Video Buffers**: Variable-size buffers for streaming data
- **Dynamic Circular Logs**: Runtime-determined log sizes
- **General Applications**: When capacity is determined at runtime

### Stack-Allocated StackArrayDeque

- **Embedded Systems**: Zero heap allocation, predictable memory usage
- **Real-time Systems**: Maximum performance, no allocation overhead
- **Fixed-size Buffers**: When capacity is known at compile time
- **ISR-safe Operations**: No heap allocation means no malloc/free in interrupts

## Comparison Table

| Feature     | `ArrayDeque`             | `StackArrayDeque`      | `VecDeque`     |
| ----------- | ------------------------ | ---------------------- | -------------- |
| Allocation  | Heap                     | Stack                  | Heap           |
| Capacity    | Runtime-fixed            | Compile-time fixed     | Dynamic, grows |
| Memory      | Predictable after init   | Completely predictable | May reallocate |
| Overflow    | Overwrites oldest        | Overwrites oldest      | Grows capacity |
| Performance | Consistent O(1)          | Consistent O(1)        | Amortized O(1) |
| No-std      | Supported (with `alloc`) | Fully supported        | Requires `std` |
| ISR-safe    | No (heap allocation)     | Yes                    | No             |

## License

This project is licensed under the [MIT License](LICENSE).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

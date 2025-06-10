# ArrayDeque

[![Crates.io](https://img.shields.io/crates/v/array-deque.svg)](https://crates.io/crates/array-deque)
[![Documentation](https://docs.rs/array-deque/badge.svg)](https://docs.rs/array-deque)
[![License](https://img.shields.io/crates/l/array-deque.svg)](https://github.com/0x7030676e31/array-deque/blob/master/LICENSE)
A fixed-capacity circular buffer (ring buffer) implementation providing efficient double-ended queue operations. Unlike `std::collections::VecDeque`, `ArrayDeque` has a compile-time fixed capacity and will overwrite old elements when full, making it ideal for scenarios where memory usage is constrained or predictable.

## Features

- **Fixed Capacity**: Memory usage is determined at creation time
- **Circular Buffer**: Efficient O(1) operations at both ends
- **Overwrite Behavior**: When full, new elements overwrite the oldest ones
- **Zero Allocations**: After initial allocation, no further memory allocations
- **Serde Support**: Optional serialization/deserialization (with `serde` feature)
- **Iterator Support**: Full iterator implementation with `IntoIterator`
- **Index Access**: Direct element access via indexing
- **Clone Support**: Deep cloning of the entire deque

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
array-deque = "0.1.0"

# For serde support
array-deque = { version = "0.1.0", features = ["serde"] }
```

## Usage

### Basic Operations

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

### Overflow Behavior

When the deque reaches its capacity, new elements will overwrite the oldest ones:

```rust
use array_deque::ArrayDeque;

let mut deque = ArrayDeque::new(3);
deque.push_back(1);
deque.push_back(2);
deque.push_back(3);

// Deque is now full: [1, 2, 3]

deque.push_back(4);  // Overwrites 1
// Deque is now: [2, 3, 4]

assert_eq!(deque.pop_front(), Some(2));
assert_eq!(deque.pop_front(), Some(3));
assert_eq!(deque.pop_front(), Some(4));
```

### Iteration

```rust
use array_deque::ArrayDeque;

let mut deque = ArrayDeque::new(5);
deque.extend(vec![1, 2, 3, 4, 5]);

// Iterate by reference
for item in &deque {
    println!("{}", item);
}

// Iterate by value (consumes the deque)
for item in deque {
    println!("{}", item);
}
```

### Creating from Collections

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

### Serde Support

With the `serde` feature enabled:

```rust
use array_deque::ArrayDeque;
use serde_json;

let mut deque = ArrayDeque::new(3);
deque.extend(vec![1, 2, 3]);

// Serialize
let json = serde_json::to_string(&deque).unwrap();
assert_eq!(json, "[1,2,3]");

// Deserialize
let deque: ArrayDeque<i32> = serde_json::from_str("[4,5,6]").unwrap();
assert_eq!(deque.len(), 3);
```

## API Overview

### Core Methods

- `new(capacity)` - Create a new deque with fixed capacity
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
- `into_iter()` - Consuming iterator

## Performance

All core operations are O(1):

- `push_back` / `push_front`: O(1)
- `pop_back` / `pop_front`: O(1)
- Index access: O(1)
- `len` / `is_empty` / `is_full`: O(1)

## Use Cases

`ArrayDeque` is particularly well-suited for:

- **Audio/Video Buffers**: Fixed-size buffers for streaming data
- **Circular Logs**: Maintaining recent log entries with automatic overflow
- **Game Development**: Entity queues, command buffers
- **Embedded Systems**: Predictable memory usage
- **Real-time Systems**: No allocation after initialization
- **Window Functions**: Sliding window algorithms

## Comparison with `VecDeque`

| Feature | `ArrayDeque` | `VecDeque` |
|---------|--------------|------------|
| Capacity | Fixed at creation | Dynamic, grows as needed |
| Memory | Predictable, no allocations after init | May reallocate and grow |
| Overflow | Overwrites oldest elements | Grows capacity |
| Performance | Consistent O(1) | Amortized O(1), occasional O(n) |
| Use Case | Fixed-size buffers, embedded | General-purpose, variable size |

## License

This project is licensed under one of the following:

- [MIT License](LICENSE)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

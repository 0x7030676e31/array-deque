#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
//! A fixed-capacity circular buffer (ring buffer) implementation.
//!
//! This crate provides [`ArrayDeque`], a double-ended queue with a fixed capacity
//! that uses a circular buffer for efficient operations at both ends. Unlike
//! [`std::collections::VecDeque`], this implementation has a compile-time fixed
//! capacity and will overwrite old elements when full.
//!
//! # Examples
//!
//! ```
//! use array_deque::ArrayDeque;
//!
//! let mut deque = ArrayDeque::new(3);
//! deque.push_back(1);
//! deque.push_back(2);
//! deque.push_back(3);
//!
//! assert_eq!(deque.pop_front(), Some(1));
//! assert_eq!(deque.pop_back(), Some(3));
//! ```
//!
//! # Features
//!
//! - **serde**: Enable serialization and deserialization support with serde.

mod array_deque;
mod stack_array_deque;

pub use array_deque::ArrayDeque;
pub use stack_array_deque::StackArrayDeque;

#![feature(const_generics)]
#![allow(incomplete_features)]

/// Varchar implements a string with fixed size in bytes.
pub mod varchar;

/// Table implements a logic to work with a file with the table data.
pub mod table;

/// TableTrait implements special methods to interact with the table to store.
pub mod table_trait;

/// TableIndex implements an index for a value in the table.
pub mod table_index;

pub use varchar::*;
pub use table::*;
pub use table_trait::*;
pub use table_index::*;

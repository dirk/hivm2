// NOTE: Macro-exporting modules must come before all others!

/// Traits, functions, and macros to make working with bytecode easier.
#[macro_use]
pub mod util;

/// The actual bytecode operations and their encoding/decoding logic.
pub mod ops;

/// The various types of data in the bytecode (register indexes, local variable indexes, etc.).
pub mod types;

//! Model, identifiers, file discovery and operations behind the `bibx` CLI.
//!
//! Operations return plans (the new file text, what was removed) and never print or write;
//! the caller decides what to do with them. Edits are byte-range splices of the original
//! text, so everything the user did not touch stays byte for byte.

pub mod bibfile;
pub mod discover;
pub mod error;
pub mod identifier;
pub mod ops;
pub mod write;

pub use bibfile::{BibFile, LineEnding};
pub use discover::{Target, resolve_files};
pub use error::{Error, Result};
pub use identifier::{Identifier, Kind};

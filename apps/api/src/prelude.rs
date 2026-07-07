// SPDX-License-Identifier: AGPL-3.0-or-later
//! Crate prelude - re-exports the error types for `use crate::prelude::*;`.

#[allow(unused_imports)] // re-exported for use as fallible paths land
pub use crate::error::{Error, Result};

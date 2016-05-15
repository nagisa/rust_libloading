use LibTracked;
use LibUnsafe;
use std::sync::Arc;

/// A shared library which uses atomic ref-counting to track its [Symbols](trait.Symbol.html).
pub type LibArc = LibTracked<Arc<LibUnsafe>>;

use LibTracked;
use LibUnsafe;
use std::rc::Rc;

/// A shared library which uses non-atomic ref-counting to track its [Symbols](trait.Symbol.html).
pub type LibRc = LibTracked<Rc<LibUnsafe>>;

use FuncTracked;
use LibUnsafe;
use std::rc::Rc;

/// A pointer to a shared function which uses non-atomic ref-counting to avoid outliving its library.
pub type FuncRc<T> = FuncTracked<T, Rc<LibUnsafe>>;

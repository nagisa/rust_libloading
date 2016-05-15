use DataTracked;
use LibUnsafe;
use std::rc::Rc;

/// A pointer to shared data which uses non-atomic ref-counting to avoid outliving its library.
pub type DataRc<T> = DataTracked<T, Rc<LibUnsafe>>;

use DataTracked;
use LibUnsafe;
use std::sync::Arc;

/// A pointer to shared data which uses atomic ref-counting to avoid outliving its library.
pub type DataArc<T> = DataTracked<T, Arc<LibUnsafe>>;

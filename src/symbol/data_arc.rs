use DataTracked;
use LibArc;

/// A pointer to shared data which uses atomic ref-counting to avoid outliving its library.
pub type DataArc<T> = DataTracked<T, LibArc>;

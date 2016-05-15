use DataTracked;
use LibRc;

/// A pointer to shared data which uses non-atomic ref-counting to avoid outliving its library.
pub type DataRc<T> = DataTracked<T, LibRc>;

use FuncTracked;
use LibArc;

/// A pointer to a shared function which uses atomic ref-counting to avoid outliving its library.
pub type FuncArc<T> = FuncTracked<T, LibArc>;

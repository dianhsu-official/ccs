use std::path::Path;

pub(crate) fn handle<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) {
    if !to.as_ref().exists() {
        std::fs::copy(from, to).unwrap();
    }
}

use std::path::Path;

pub(crate) fn handle<P: AsRef<Path> + Copy, Q: AsRef<Path> + Copy>(from: P, to: Q) {
    if !to.as_ref().exists() {
        match std::fs::copy(from, to) {
            Ok(_) => {
                log::info!(
                    "Copy template file from {} to {}.",
                    from.as_ref().display(),
                    to.as_ref().display()
                );
            }
            Err(err) => {
                log::error!(
                    "Failed to copy template file from {} to {}: {}",
                    from.as_ref().display(),
                    to.as_ref().display(),
                    err
                );
            }
        }
    } else {
        log::info!(
            "{} already exists, skip copy template file.",
            to.as_ref().display()
        );
    }
}

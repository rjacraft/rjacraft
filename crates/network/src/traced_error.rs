use std::{error, fmt};

pub struct TracedError<T>(pub T);

impl<T: error::Error> fmt::Display for TracedError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut current = &self.0 as &dyn error::Error;

        loop {
            write!(f, "    > {}", current)?;

            if let Some(new) = current.source() {
                write!(f, "\n")?;
                current = new;
            } else {
                break;
            }
        }

        Ok(())
    }
}

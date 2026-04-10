//! [`Report`] — textual report for devices, rooms, and the whole home.

/// Types that can produce a human-readable report string.
pub trait Report {
    /// Full report text (typically multiple lines).
    fn report(&self) -> String;

    /// Prints the report to stdout.
    fn print_report(&self) {
        print!("{}", self.report());
    }
}

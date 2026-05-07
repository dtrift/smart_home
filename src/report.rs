//! [`Report`] — textual report for devices, rooms, and the whole home.

use std::marker::PhantomData;

/// Types that can produce a human-readable report string.
pub trait Report {
    /// Full report text (typically multiple lines).
    fn report(&self) -> String;

    /// Prints the report to stdout.
    fn print_report(&self) {
        print!("{}", self.report());
    }
}

/// Static-polymorphism report composer for heterogeneous [`Report`] values.
pub struct Reporter<'a, Items = ()> {
    items: Items,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Reporter<'a, ()> {
    pub fn new() -> Self {
        Self {
            items: (),
            _marker: PhantomData,
        }
    }
}

impl Default for Reporter<'_, ()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Items> Reporter<'a, Items> {
    #[allow(clippy::should_implement_trait)]
    pub fn add<T>(self, item: &'a T) -> Reporter<'a, (Items, &'a T)>
    where
        T: Report + 'a,
    {
        Reporter {
            items: (self.items, item),
            _marker: PhantomData,
        }
    }
}

impl<Items> Reporter<'_, Items>
where
    Items: ReportItems,
{
    pub fn report(&self) -> String {
        let mut output = String::new();
        self.items.append_report(&mut output);
        print!("{output}");
        output
    }
}

pub trait ReportItems {
    fn append_report(&self, output: &mut String);
}

impl ReportItems for () {
    fn append_report(&self, _output: &mut String) {}
}

impl<Items, T> ReportItems for (Items, &T)
where
    Items: ReportItems,
    T: Report,
{
    fn append_report(&self, output: &mut String) {
        self.0.append_report(output);
        output.push_str(&self.1.report());
    }
}

pub mod statistics_format;

pub trait FormatEmbedded<'a, T> {
    fn format(&'a mut self, value: &T) -> &'a mut Self;
}

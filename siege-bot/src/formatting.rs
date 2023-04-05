mod all_maps_format;
mod all_operators_format;
mod statistics_format;

pub trait FormatEmbedded<'a, T> {
    fn format(&'a mut self, value: &T) -> &'a mut Self;
}

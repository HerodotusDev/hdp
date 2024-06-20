pub trait AsCairoFormat {
    type Output;

    fn as_cairo_format(&self) -> Self::Output;
}

pub trait IntoFelts {
    type Output;

    fn to_felts(&self) -> Self::Output;
}

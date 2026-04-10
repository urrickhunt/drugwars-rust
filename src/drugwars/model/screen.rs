#[derive(Debug, Clone, Copy)]
pub struct ScreenState {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) half_width: usize,
}

impl ScreenState {
    pub(crate) const fn new() -> Self {
        Self {
            width: 80,
            height: 24,
            half_width: 40,
        }
    }
}

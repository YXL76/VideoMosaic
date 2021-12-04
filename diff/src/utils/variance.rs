/// iteratively compute variance
/// See [Recursive formula for variance](https://math.stackexchange.com/questions/374881/recursive-formula-for-variance)
#[derive(Default, Copy, Clone)]
pub(crate) struct Variance {
    n: i64,
    m: i64,
    s: f32,
}

impl Variance {
    #[inline(always)]
    pub(crate) fn new() -> Self {
        Default::default()
    }

    #[inline(always)]
    pub(crate) fn next(&mut self, x: i64) {
        if self.n > 0 {
            self.s += (self.n * x - self.m).pow(2) as f32 / (self.n * (self.n + 1)) as f32;
        }
        self.m += x;
        self.n += 1;
    }

    /* #[inline(always)]
    pub(crate) fn mean(&self) -> f32 {
        self.m as f32 / self.n as f32
    } */

    #[inline(always)]
    pub(crate) fn variance(&self) -> f32 {
        self.s / self.n as f32
    }
}

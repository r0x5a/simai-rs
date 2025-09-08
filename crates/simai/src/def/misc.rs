#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bpm(pub f64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Div(pub u32);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DivAbs(pub f64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tick(pub u32);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PseudoTick(pub u32);

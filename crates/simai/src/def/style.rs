use bitflags::Flags;
use serde::{Deserialize, Serialize};

pub type S = u8;

pub const NONE: S = 0;
pub const BREAK: S = 1;
pub const EX: S = 2;
pub const NAKED_STAR: S = 4;
pub const TAP_STAR: S = 8;
pub const FIREWORK: S = 16;
pub const SUDDEN: S = 32;
pub const REMOVE: S = 64;

bitflags::bitflags! {
	#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
	#[serde(transparent)]
	pub struct TapStyle: S { const _ = BREAK | EX | NAKED_STAR; }

	#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
	#[serde(transparent)]
	pub struct HoldStyle: S { const _ = BREAK | EX; }

	#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
	#[serde(transparent)]
	pub struct StarStyle: S { const _ = BREAK | EX | TAP_STAR | REMOVE | SUDDEN; }

	#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
	#[serde(transparent)]
	pub struct SlideStyle: S { const _ = BREAK; }

	#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
	#[serde(transparent)]
	pub struct TouchStyle: S { const _ = FIREWORK; }
}

pub fn to_style(c: char) -> S {
	match c {
		'b' => BREAK,
		'x' => EX,
		'$' => NAKED_STAR,
		'@' => TAP_STAR,
		'f' => FIREWORK,
		'!' => SUDDEN,
		'?' => REMOVE,
		_ => panic!("invalid style character: {}", c),
	}
}

pub(crate) fn merge<T: Flags<Bits = S>>(v: &[S]) -> T {
	let mut style = NONE;
	for &s in v {
		style |= s;
	}
	T::from_bits(style).unwrap()
}

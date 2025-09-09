use fraction::BigFraction;

use crate::def::{HoldStyle, SlideStyle, StarStyle, TapStyle, TouchStyle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[rustfmt::skip]
pub enum Key { K1 = 1, K2, K3, K4, K5, K6, K7, K8 }

impl From<char> for Key {
	fn from(c: char) -> Self {
		match c {
			'1' => Key::K1,
			'2' => Key::K2,
			'3' => Key::K3,
			'4' => Key::K4,
			'5' => Key::K5,
			'6' => Key::K6,
			'7' => Key::K7,
			'8' => Key::K8,
			_ => unreachable!(),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[rustfmt::skip]
pub enum SensorGroup { A, B, C, D, E }

impl From<char> for SensorGroup {
	fn from(c: char) -> Self {
		match c {
			'A' => SensorGroup::A,
			'B' => SensorGroup::B,
			'C' => SensorGroup::C,
			'D' => SensorGroup::D,
			'E' => SensorGroup::E,
			_ => unreachable!(),
		}
	}
}

pub type Frac = BigFraction;

#[derive(Debug, Clone, PartialEq)]
pub enum Len {
	Rel(Frac),
	Bpm { bpm: f64, frac: Frac },
	Abs(f64),
	Zero,
}

impl Len {
	pub fn bpm(bpm: f64, p: u32, q: u32) -> Self {
		Len::Bpm { bpm, frac: Frac::new(q, p) }
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum Wait {
	Rel,
	Bpm(f64),
	Abs(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
	Line,
	ArcLeft,
	ArcRight,
	Arc,
	P,
	Q,
	S,
	Z,
	PP,
	QQ,
	V,
	Fan,
	Angle(Key),
}

impl From<char> for Shape {
	fn from(c: char) -> Self {
		match c {
			'-' => Shape::Line,
			'<' => Shape::ArcLeft,
			'>' => Shape::ArcRight,
			'^' => Shape::Arc,
			'p' => Shape::P,
			'q' => Shape::Q,
			's' => Shape::S,
			'z' => Shape::Z,
			'v' => Shape::V,
			'w' => Shape::Fan,
			_ => unreachable!(),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sensor {
	pub group: SensorGroup,
	pub index: Option<Key>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tap {
	pub key: Key,
	pub style: TapStyle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Hold {
	pub key: Key,
	pub len: Len,
	pub style: HoldStyle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TouchTap {
	pub sensor: Sensor,
	pub style: TouchStyle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TouchHold {
	pub sensor: Sensor,
	pub len: Len,
	pub style: TouchStyle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Slide {
	pub key: Key,
	pub star_style: StarStyle,
	pub tracks: Vec<SlideTrack>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SlideTrack {
	Amortized { path: Vec<(Shape, Key)>, wait: Wait, style: SlideStyle, len: Len },
	Piecewise { path: Vec<(Shape, Key, Len)>, wait: Wait, style: SlideStyle },
}

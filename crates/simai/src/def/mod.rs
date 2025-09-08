mod misc;
mod note;
mod style;

pub use misc::*;
pub use note::*;
pub use style::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
	Bpm(Bpm),
	Div(Div),
	DivAbs(DivAbs),

	Tap(Tap),
	Hold(Hold),
	TouchTap(TouchTap),
	TouchHold(TouchHold),
	Slide(Slide),

	Tick(Tick),
	PseudoTick(PseudoTick),

	Comment(String),
	End,
}

// 	Bpm(f64),
// 	Div(u32),
// 	DivAbs(f64),

// 	Tap { key: Key, style: TapStyle },
// 	Hold { key: Key, style: HoldStyle, len: Len },
// 	TouchTap { key: Sensor, style: TouchStyle },
// 	TouchHold { key: Sensor, style: TouchStyle, len: Len },
// 	Slide { key: Key, star_style: StarStyle, style: SlideStyle, tracks: Vec<SlidePath> },

// 	Tick(u32),
// 	PseudoTick(u32),

// 	Comment(String),
// 	End,
// }

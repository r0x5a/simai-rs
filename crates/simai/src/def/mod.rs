use chumsky::span::SimpleSpan;

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

	Comment(String), // TODO: these are currently not parsed
	End,

	Error(SimpleSpan), // TODO
}

use chumsky::prelude::*;

use crate::def::*;

macro_rules! make_styles {
	($t:ty, $s:expr) => {
		one_of($s).map(to_style).padded().repeated().collect().map(|v: Vec<_>| merge::<$t>(&v))
	};
}

// TODO: also return a lenient parser
pub fn simai<'a>() -> impl Parser<'a, &'a str, Vec<Item>> {
	let sym = |c| just(c).padded();

	// common parsers
	let digits = text::digits(10);
	let int = digits.to_slice().map(|s: &str| s.parse::<u32>().unwrap()).padded();
	let float = digits
		.then(sym('.').ignore_then(digits.or_not()).or_not())
		.to_slice()
		.map(|s: &str| s.replace(' ', "").parse::<f64>().unwrap())
		.padded();

	// key and sensor
	let key = one_of('1'..='8').map(|c: char| c.into()).padded();
	let sensor = choice((
		one_of("ABDE")
			.then(key.clone())
			.map(|(g, i): (char, _)| Sensor { group: g.into(), index: Some(i) }),
		sym('C')
			.ignore_then(one_of("12").or_not())
			.map(|i| Sensor { group: SensorGroup::C, index: i.map(Key::from) }),
	))
	.padded();

	// len and slide wait
	let len = choice((
		sym('#').ignore_then(float).map(Len::Abs),
		int.then_ignore(sym(':')).then(int).map(|(p, q)| Len::Rel(Frac::new(q, p))),
		float
			.then_ignore(sym('#'))
			.then(int)
			.then_ignore(sym(':'))
			.then(int)
			.map(|((bpm, p), q)| Len::Bpm { bpm, frac: Frac::new(q, p) }),
	))
	.delimited_by(sym('['), sym(']'))
	.or(text::whitespace().to(Len::Zero));

	// let wait_head =

	// prefix items
	let bpm = float.delimited_by(sym('('), sym(')')).map(|n| Item::Bpm(Bpm(n)));
	let div = int.delimited_by(sym('{'), sym('}')).map(|n| Item::Div(Div(n)));
	let div_abs =
		float.delimited_by(sym('{').then(sym('#')), sym('}')).map(|n| Item::DivAbs(DivAbs(n)));

	// notes
	let tap_styles = make_styles!(TapStyle, "bx$");
	let tap = key.clone().then(tap_styles).map(|(key, style)| Item::Tap(Tap { key, style }));

	let touch_styles = make_styles!(TouchStyle, "f");
	let touch_tap = (sensor.clone())
		.then(touch_styles)
		.map(|(sensor, style)| Item::TouchTap(TouchTap { sensor, style }));

	let hold_styles = make_styles!(HoldStyle, "bx");
	let hold = (key.clone())
		.then(hold_styles)
		.then_ignore(sym('h'))
		.then(hold_styles)
		.then(len.clone())
		.then(hold_styles)
		.map(|((((key, s1), s2), l), s3)| Item::Hold(Hold { key, len: l, style: s1 | s2 | s3 }));

	let touch_hold = sensor
		.then(touch_styles)
		.then_ignore(sym('h'))
		.then(touch_styles)
		.then(len)
		.then(touch_styles)
		.map(|((((sensor, s1), s2), l), s3)| {
			Item::TouchHold(TouchHold { sensor, len: l, style: s1 | s2 | s3 })
		});

	// comment and end
	let comment = (just("||").padded())
		.ignore_then(none_of("\r\n").repeated().at_least(0).collect::<String>())
		.map(Item::Comment);
	let comments = comment.repeated().at_least(1).collect();
	let end = sym('E').to(Item::End);
	let suf_items = choice((end.map(I::Item), comments.map(I::Items)));

	// tap group
	#[derive(Clone)]
	enum I {
		Item(Item),
		Items(Vec<Item>),
	}
	impl From<I> for Vec<Item> {
		fn from(v: I) -> Self {
			match v {
				I::Item(it) => vec![it],
				I::Items(its) => its,
			}
		}
	}
	let tap_group = key
		.map(|key| Item::Tap(Tap { key, style: TapStyle::empty() }))
		.repeated()
		.at_least(2) // Cannot be 1, because that would be ambiguous with tap
		.collect::<Vec<_>>();

	// note item
	// prio: hold > tap group > tap
	let note_item = choice((
		choice((hold, touch_hold)).map(I::Item),
		tap_group.map(I::Items),
		choice((tap, touch_tap)).map(I::Item),
	))
	.boxed();
	let first_note_item = (note_item.clone()).map(Vec::from);
	let note_items =
		first_note_item.foldl(sym('/').ignore_then(note_item).repeated(), |mut acc, item| {
			match item {
				I::Item(it) => acc.push(it),
				I::Items(its) => acc.extend(its),
			}
			acc
		});

	let pre_items = choice((
		bpm.then(div).map(|(b, d)| I::Items(vec![b, d])),
		bpm.map(I::Item),
		div.map(I::Item),
		div_abs.map(I::Item),
	));
	let note_items = pre_items.or_not().then(note_items.or_not()).then(suf_items.or_not()).map(
		|((pre, notes), suf)| {
			let mut v = pre.map(Vec::from).unwrap_or_default();
			if let Some(notes) = notes {
				v.extend(notes);
			}
			if let Some(suf) = suf {
				match suf {
					I::Item(it) => v.push(it),
					I::Items(its) => v.extend(its),
				}
			}
			v
		},
	);

	// misc
	let tick = sym(',').repeated().at_least(1).count().map(|v| Item::Tick(Tick(v as u32)));
	let pseudo_tick =
		sym('`').repeated().at_least(1).count().map(|v| Item::PseudoTick(PseudoTick(v as u32)));

	let tick_item = choice((tick, pseudo_tick));

	// full parser
	let comments_or_not = comment.repeated().at_least(0).collect::<Vec<_>>();
	let header = comments_or_not.then(pre_items).then(comments_or_not).map(|((a, b), c)| {
		let mut v = a;
		match b {
			I::Item(it) => v.push(it),
			I::Items(its) => v.extend(its),
		}
		v.extend(c);
		v
	});

	header.foldl(tick_item.then(note_items).repeated(), |mut acc, (t, mut notes)| {
		acc.push(t);
		acc.append(&mut notes);
		acc
	})
}

pub fn test<'a>() -> impl Parser<'a, &'a str, Len> {
	let sym = |c| just(c).padded();

	let int = text::digits(10).to_slice().map(|s: &str| s.parse::<u32>().unwrap()).padded();
	let float = text::digits(10)
		.then(sym('.').ignore_then(text::digits(10).or_not()).or_not())
		.to_slice()
		.map(|s: &str| s.replace(' ', "").parse::<f64>().unwrap())
		.padded();

	choice((
		sym('#').ignore_then(float).map(Len::Abs),
		int.then_ignore(sym(':')).then(int).map(|(p, q)| Len::Rel(Frac::new(q, p))),
		float
			.then_ignore(sym('#'))
			.then(int)
			.then_ignore(sym(':'))
			.then(int)
			.map(|((bpm, p), q)| Len::Bpm { bpm, frac: Frac::new(q, p) }),
	))
	.delimited_by(sym('['), sym(']'))
	.or(text::whitespace().to(Len::Zero))
}

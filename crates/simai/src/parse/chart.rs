use chumsky::{extra::Err, prelude::*};

use crate::def::*;

static CHAR_LIST: &str = "12345678ABCDE-<>^szvwpqV/,`[]*h{}()E \t\n\r";

macro_rules! make_styles {
	($t:ty, $s:expr) => {
		choice((
			one_of($s).map(|c| Some(to_style(c))),
			none_of::<_, _, Err<Rich<char>>>(CHAR_LIST).validate(|c, e, emitter| {
				emitter.emit(Rich::custom(
					e.span(),
					format!("Invalid style modifier '{}' for {}", c, stringify!($t)),
				));
				None
			}),
		))
		.padded()
		.repeated()
		.collect::<Vec<_>>()
		.map(|v| {
			let valid_styles: Vec<_> = v.into_iter().flatten().collect();
			merge::<$t>(&valid_styles)
		})
		.labelled(stringify!($t))
	};
}

// TODO: also return a lenient parser
// comments are not handled here. use a preprocessor to remove comments
pub fn simai<'a>() -> impl Parser<'a, &'a str, Vec<Item>, Err<Rich<'a, char>>> {
	let sym = |c| just(c).padded();
	let sym2 = |c| just(c).padded();

	// common parsers
	let digits = text::digits(10).labelled("digits");
	let int = digits.to_slice().map(|s: &str| s.parse::<u32>().unwrap()).padded().labelled("int");
	let float = digits
		.then(sym('.').ignore_then(digits.or_not()).or_not())
		.to_slice()
		.map(|s: &str| s.replace(' ', "").parse::<f64>().unwrap())
		.padded()
		.labelled("float");

	// key and sensor
	let key = one_of('1'..='8').map(|c: char| c.into()).padded().labelled("key");
	let sensor = choice((
		one_of("ABDE")
			.then(key.clone())
			.map(|(g, i): (char, _)| Sensor { group: g.into(), index: Some(i) }),
		sym('C')
			.ignore_then(one_of("12").or_not())
			.map(|i| Sensor { group: SensorGroup::C, index: i.map(Key::from) }),
	))
	.padded()
	.labelled("sensor");

	// tap and touch tap
	let tap_styles = make_styles!(TapStyle, "bx$");
	let tap = key.clone().then(tap_styles).map(|(key, style)| Item::Tap(Tap { key, style }));

	let touch_styles = make_styles!(TouchStyle, "f");
	let touch_tap = (sensor.clone())
		.then(touch_styles)
		.map(|(sensor, style)| Item::TouchTap(TouchTap { sensor, style }));

	// len
	let frac = int.then_ignore(sym(':')).then(int).map(|(p, q)| Frac::new(q, p));
	let len_abs = float.map(Len::Abs);
	let len_rel = frac.map(Len::Rel);
	let len_bpm = float.then_ignore(sym('#')).then(frac).map(|(bpm, frac)| Len::Bpm { bpm, frac });
	let len = choice((sym('#').ignore_then(len_abs), len_rel, len_bpm))
		.delimited_by(sym('['), sym(']'))
		.boxed();
	let len_or_zero = len.clone().or(empty().to(Len::Zero));

	// hold and touch hold
	let hold_styles = make_styles!(HoldStyle, "bx");
	let hold = (key.clone())
		.then(hold_styles)
		.then_ignore(sym('h'))
		.then(hold_styles)
		.then(len_or_zero.clone())
		.then(hold_styles)
		.map(|((((key, s1), s2), len), s3)| Item::Hold(Hold { key, len, style: s1 | s2 | s3 }));

	let touch_hold = sensor
		.then(touch_styles)
		.then_ignore(sym('h'))
		.then(touch_styles)
		.then(len_or_zero.clone())
		.then(touch_styles)
		.map(|((((sensor, s1), s2), len), s3)| {
			Item::TouchHold(TouchHold { sensor, len, style: s1 | s2 | s3 })
		});

	// slide wait
	let wait_rel = len_rel.map(|f| (Wait::Rel, f));
	let wait_bpm = float
		.then_ignore(sym('#'))
		.then(choice((len_rel, len_abs)))
		.map(|(bpm, len)| (Wait::Bpm(bpm), len));
	let wait_abs = float
		.then_ignore(sym2("##"))
		.then(choice((len_rel, len_abs, len_bpm)))
		.map(|(time, len)| (Wait::Abs(time), len));
	let wait_any = choice((wait_rel, wait_bpm, wait_abs)).delimited_by(sym('['), sym(']')).boxed();

	// shape
	let shape = choice((
		sym2("pp").to(Shape::PP),
		sym2("qq").to(Shape::QQ),
		one_of("-<>^szvwpq").map(Shape::from),
		sym('V').ignore_then(key.clone()).map(Shape::Angle),
	))
	.padded()
	.labelled("shape");

	// slide:
	// - amortized: A-B-C-D(style?)[any wait]
	// - piecewise: A-B[any wait]-C[hold len]-D(style?)[hold len]
	//
	// the actual parsing logic (prio from top to bottom):
	// - piecewise:		slide_ext wait_any (slide_ext len)* (slide_ext_styled len)? style?
	// - amortized:		slide_ext+ style? wait_any style?
	// These parsing logic avoided the '*' and '+' operators early consuming too much input.
	// Single-segment slides are parsed as piecewise, but should be mapped to amortized.

	let star_styles = make_styles!(StarStyle, "bx@?!");
	let slide_styles = make_styles!(SlideStyle, "b");

	let slide_ext = shape.clone().then(key.clone());
	let slide_ext_styled = group((shape.clone(), key.clone(), slide_styles));

	let slide_track_piecewise = group((
		slide_ext.clone(),
		wait_any.clone(),
		slide_ext.clone().then(len.clone()).map(|((a, b), c)| (a, b, c)).repeated().collect::<Vec<_>>(),
		slide_ext_styled.then(len).or_not(),
		slide_styles,
	))
	.map(|((shape, key), (wait, len), middle, last, mut style)| {
		if middle.is_empty() && last.is_none() {
			return SlideTrack::Amortized { path: vec![(shape, key)], wait, style, len };
		}

		let mut path = vec![(shape, key, len)];
		path.extend(middle);
		if let Some(((shape, key, s), len)) = last {
			path.push((shape, key, len));
			style |= s;
		}
		SlideTrack::Piecewise { path, wait, style }
	});

	let slide_track_amortized = group((
		slide_ext.repeated().at_least(1).collect::<Vec<_>>(),
		slide_styles,
		wait_any,
		slide_styles,
	))
	.map(|(path, s1, (wait, len), s2)| SlideTrack::Amortized { path, wait, style: s1 | s2, len });

	let slide_track = choice((slide_track_piecewise, slide_track_amortized)).boxed();

	let slide = (key.clone())
		.then(star_styles)
		.then(slide_track.separated_by(sym('*')).at_least(1).collect())
		.map(|((key, star_style), tracks)| Item::Slide(Slide { key, star_style, tracks }));

	// prefix items (bpm, div, div abs)
	let bpm = float.delimited_by(sym('('), sym(')')).map(|n| Item::Bpm(Bpm(n)));
	let div = int.delimited_by(sym('{'), sym('}')).map(|n| Item::Div(Div(n)));
	let div_abs =
		float.delimited_by(sym('{').then(sym('#')), sym('}')).map(|n| Item::DivAbs(DivAbs(n)));

	// end mark
	let end_mark = sym('E').to(Item::End);

	// tap group
	#[derive(Debug, Clone)]
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
	// prio: hold > tap group > slide > tap
	let note_item = choice((
		choice((hold, touch_hold)).map(I::Item),
		tap_group.map(I::Items),
		slide.map(I::Item),
		choice((tap, touch_tap)).map(I::Item),
	))
	.boxed();
	let first_note_item = note_item.clone().map(Vec::from);

	let slash_recovery =
		none_of(",`/").repeated().at_least(1).to_span().then_ignore(sym('/')).map(Some);
	let slash = sym('/').to(None).recover_with(via_parser(slash_recovery));

	let note_items = choice((
		first_note_item.foldl(slash.then(note_item).repeated(), |mut acc, (slash, item)| {
			if let Some(err) = slash {
				acc.push(Item::Error(err));
			}
			match item {
				I::Item(it) => acc.push(it),
				I::Items(its) => acc.extend(its),
			}
			acc
		}),
		end_mark.to(vec![Item::End]),
	))
	.labelled("notes");

	let pre_items = choice((
		bpm.then(div).map(|(b, d)| I::Items(vec![b, d])),
		bpm.map(I::Item),
		div.map(I::Item),
		div_abs.map(I::Item),
	))
	.boxed();
	let main_items =
		pre_items.clone().or_not().then(note_items.clone().or_not()).map(|(pre, notes)| {
			let mut v = pre.map(Vec::from).unwrap_or_default();
			if let Some(notes) = notes {
				v.extend(notes);
			}
			v
		});

	// misc
	let tick = sym(',').repeated().at_least(1).count().map(|v| Item::Tick(Tick(v as u32)));
	let pseudo_tick =
		sym('`').repeated().at_least(1).count().map(|v| Item::PseudoTick(PseudoTick(v as u32)));

	let tick_item = choice((tick, pseudo_tick));
	let tick_recovery = (none_of(",`").repeated().at_least(1).to_span())
		.then(tick_item.map(Some).or(end().to(None)))
		.map(|(err, t)| match t {
			Some(t) => I::Items(vec![Item::Error(err), t]),
			None => I::Item(Item::Error(err)),
		});
	let tick_item = tick_item.map(I::Item).recover_with(via_parser(tick_recovery));

	// full parser
	let header = pre_items.then(note_items.or_not()).map(|(pre, notes)| {
		let mut v: Vec<_> = pre.into();
		if let Some(notes) = notes {
			v.extend(notes);
		}
		v
	});

	header.foldl(tick_item.then(main_items).repeated(), |mut acc, (t, mut notes)| {
		match t {
			I::Item(it) => acc.push(it),
			I::Items(its) => acc.extend(its),
		}
		acc.append(&mut notes);
		acc
	})
}

pub fn rm_comments(s: &str) -> String {
	s.lines()
		.map(|line| if let Some(pos) = line.find("||") { &line[..pos] } else { line })
		.collect::<Vec<_>>()
		.join("\n")
}

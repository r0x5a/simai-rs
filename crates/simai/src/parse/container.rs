use std::{collections::HashMap, str::FromStr};

use crate::def::Item;

#[derive(Debug, Clone, Default)]
pub struct Simai {
	title: Option<String>,
	artist: Option<String>,
	first: Option<f64>,
	rest_cmds: HashMap<String, String>,

	designer: [Option<String>; 8],
	level: [Option<String>; 8],
	chart: [Option<Vec<Item>>; 8],
}

impl Simai {
	pub fn new() -> Self {
		Default::default()
	}

	fn append_cmd(&mut self, cmd: String, value: String) {
		let s = cmd.as_str();

		macro_rules! parse_diff {
			($i:expr) => {{
				use chumsky::Parser;
				match s {
					concat!("des_", stringify!($i)) => {
						self.designer[$i] = Some(value);
						return;
					}
					concat!("lv_", stringify!($i)) => {
						self.level[$i] = Some(value);
						return;
					}
					concat!("inote_", stringify!($i)) => {
						println!("Parsing inote_{}", $i);
						let s = value.trim();
						if s.is_empty() {
							return;
						}
						let s = crate::parse::chart::rm_comments(&s);
						self.chart[$i] = Some(crate::parse::chart::simai().parse(&s).unwrap()); // TODO: error handling
						return;
					}
					_ => {}
				}
			}};
		}

		parse_diff!(1);
		parse_diff!(2);
		parse_diff!(3);
		parse_diff!(4);
		parse_diff!(5);
		parse_diff!(6);
		parse_diff!(7);

		match s {
			"title" => self.title = Some(value),
			"artist" => self.artist = Some(value),
			"first" => {
				let trimmed = value.trim();
				if trimmed.is_empty() {
					return;
				}
				if let Ok(first) = value.trim().parse::<f64>() {
					self.first = Some(first);
				} else {
					// TODO
				}
			}
			_ => {
				self.rest_cmds.insert(cmd, value);
			}
		}
	}
}

impl FromStr for Simai {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut simai = Simai::new();
		let mut cur_cmd: Option<String> = None;
		let mut cur_value = String::new();

		for line in s.lines() {
			if line.starts_with('&')
				&& let Some(i) = line.find('=')
			{
				if let Some(cmd) = cur_cmd.take() {
					simai.append_cmd(cmd, cur_value);
				}
				cur_cmd = Some(line[1..i].trim().to_string());
				cur_value = line[i + 1..].trim().to_string();
			} else {
				cur_value += "\n";
				cur_value += line.trim();
				continue;
			}
		}
		if let Some(cmd) = cur_cmd.take() {
			simai.append_cmd(cmd, cur_value);
		}

		Ok(simai)
	}
}

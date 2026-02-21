use std::{collections::HashMap, convert::Infallible, str::FromStr};

use chumsky::{Parser, error::Rich};

use crate::def::Item;

type ParseResult = (Option<Vec<Item>>, Vec<Rich<'static, char>>);

#[derive(Debug, Clone, Default)]
pub struct Simai {
	pub title: Option<String>,
	pub artist: Option<String>,
	pub first: Option<f64>,
	pub rest_cmds: HashMap<String, String>,

	pub designer: [Option<String>; 8],
	pub level: [Option<String>; 8],
	pub chart: [Option<ParseResult>; 8],
}

impl Simai {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn parse_chart(chart: &str) -> ParseResult {
		let s = crate::parse::chart::rm_comments(chart);
		let result = crate::parse::chart::simai().parse(&s);
		let output = result.output();
		let errors = result.errors().map(|x| x.clone().into_owned()).collect::<Vec<_>>();

		(output.cloned(), errors)
	}

	fn append_cmd(&mut self, cmd: String, value: String) {
		let s = cmd.as_str();

		macro_rules! parse_diff {
			($i:expr) => {{
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
						let s = value.trim();
						if s.is_empty() {
							return;
						}
						self.chart[$i] = Some(Self::parse_chart(s));
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
	type Err = Infallible;

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

use std::{collections::HashMap, convert::Infallible, str::FromStr};

use chumsky::{Parser, error::Rich};

use crate::def::Item;

#[derive(Debug, Clone, Default)]
pub struct Simai {
	pub title: Option<String>,
	pub artist: Option<String>,
	pub first: Option<f64>,
	pub rest_cmds: HashMap<String, String>,

	pub designer: [Option<String>; 8],
	pub level: [Option<String>; 8],
	pub chart: [Option<Chart>; 8],
}

#[derive(Debug, Clone)]
pub struct Chart {
	pub notes: Option<Vec<Item>>,
	pub errors: Vec<Rich<'static, char>>,
	pub raw: String,
}

impl FromStr for Chart {
	type Err = Infallible;

	fn from_str(raw: &str) -> Result<Self, Self::Err> {
		let (stripped, _) = process_comments(raw);
		let result = crate::parse::chart::simai().parse(&stripped);
		let output = result.output().cloned();
		let errors = result.errors().cloned().map(|x| x.into_owned()).collect::<Vec<_>>();

		Ok(Chart { notes: output, errors, raw: stripped })
	}
}

impl Simai {
	pub fn new() -> Self {
		Default::default()
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
						self.chart[$i] = Some(s.parse().unwrap());
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

pub fn process_comments(input: &str) -> (String, Vec<&str>) {
	let mut stripped = String::with_capacity(input.len());
	let mut comments = Vec::new();
	let mut cur = 0;

	while let Some(offset) = input[cur..].find("||") {
		let start = cur + offset;
		stripped.push_str(&input[cur..start]);

		let suffix = &input[(start + 2)..];
		let len = suffix.find(['\n', '\r']).unwrap_or(suffix.len()) + 2;

		let end = start + len;
		comments.push(&input[(start + 2)..end]);
		stripped.push_str(&" ".repeat(len));
		cur = end;
	}

	stripped.push_str(&input[cur..]);
	(stripped, comments)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_process_comments() {
		let input = "This is a test.||This is a comment.\r\nThis is another line.||Another comment.\nShould work for all line endings.||and comments at the end.";
		let (replaced, comments) = process_comments(input);
		assert_eq!(
			replaced,
			"This is a test.                    \r\nThis is another line.                  \nShould work for all line endings.                          "
		);
		assert_eq!(comments, vec!["This is a comment.", "Another comment.", "and comments at the end."]);
	}
}

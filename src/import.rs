use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::error;
use crate::graph::Graph;

struct TextFile {
	file: File,
}

impl TextFile {
	fn open(fpath: &str) -> error::Result<Self> {
		Ok(TextFile { file: File::open(fpath)?, })
	}
}

/// 解析某行
///
/// 这个函数会找到所有用双引号包裹的字符串, 然后使用 `f` 就地进行操作.
fn parse_a_line<F>(line: &String, line_num: usize, mut f: F) -> error::Result<()>
	where F: FnMut(&String) -> error::Result<()>
{
	let mut word = String::new();
	let mut has_left_quota = false;
	let mut has_backslashes = false;

	for char in line.chars() {
		let should_do = has_left_quota;

		if char == '\\' {
			has_backslashes = true;
			continue;
		}

		if has_left_quota {
			word.push(char);
		}

		if char == '"' {
			if has_backslashes {
				has_backslashes = false;
			} else {
				has_left_quota = !has_left_quota;
			}
		}

		if should_do && !has_left_quota {
			word.pop();
			f(&word)?;
			word.clear();
		}
	}
	if has_left_quota {
		return Err(error::Error::FileSyntaxWrong(
		                                         line_num,
		                                         String::from(
			"quotation marks not match",
		),
		));
	}

	Ok(())
}

fn parse_exits_line_and_insert(g: &mut Graph, exits_line: String) -> error::Result<()> {
	return parse_a_line(&exits_line, 1, |id| {
		g.new_vert(id, true);
		Ok(())
	});
}

fn parse_edge_line_and_insert(g: &mut Graph, edge_line: String, line_num: usize)
                              -> error::Result<()> {
	// 引号计数
	let mut cnt = 0;
	let mut edge: [String; 2] = [String::new(), String::new()];
	parse_a_line(&edge_line, line_num, |word| {
		match cnt {
			0 | 1 => {
				g.new_vert(word, false);
				edge[cnt] = word.clone();
				// TODO: ^ clone 造成的性能损失
			},
			2 => {
				let dist = word.parse().map_err(|_| {
					                        error::Error::FileSyntaxWrong(line_num, String::from("distance type wrong"))
				                        })?;
				g.new_edge(&edge[0], &edge[1], dist)?;
			},
			_ => {
				return Err(error::Error::FileSyntaxWrong(
				                                         line_num,
				                                         String::from("too many words"),
				));
			},
		}
		cnt += 1;
		Ok(())
	})
}

impl<'a> TryFrom<TextFile> for Graph<'a> {
	type Error = error::Error;

	fn try_from(value: TextFile) -> Result<Self, Self::Error> {
		let mut g = Graph::new();

		let buf = BufReader::new(value.file);
		let mut line_iter = buf.lines().into_iter().enumerate();

		// 第一行储存的是出口
		// 将它们提前添加到图里
		let (_, exits) = line_iter.next()
		                          .ok_or(error::Error::FileSyntaxWrong(1, String::from("file is empty")))?;
		parse_exits_line_and_insert(&mut g, exits?)?;

		// 开始处理含有顶点关系和距离的字段
		while let Some((lnum, line)) = line_iter.next() {
			let line = line?;

			if line.is_empty() {
				continue;
			}

			parse_edge_line_and_insert(&mut g, line, lnum + 1)?;
		}
		return Ok(g);
	}
}

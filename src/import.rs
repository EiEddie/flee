use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::error::{self, Error};
use crate::graph::Graph;

struct TextFile {
	file: File,
}

impl TextFile {
	fn open(fpath: &str) -> Result<Self, Error> {
		Ok(TextFile { file: File::open(fpath)?, })
	}
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
		let mut has_left_quota = false;
		let mut a_exit = String::new();
		for char in exits?.chars() {
			let should_add_vert = has_left_quota;

			if has_left_quota {
				a_exit.push(char);
			}

			if char == '\\' {
				has_left_quota = !has_left_quota;
			}

			if char == '"' {
				has_left_quota = !has_left_quota;
			}

			if should_add_vert && !has_left_quota {
				a_exit.pop();
				g.new_vert(&a_exit, true);
				a_exit.clear();
			}
		}
		if has_left_quota {
			return Err(error::Error::FileSyntaxWrong(
				1,
				String::from("quotation marks not match"),
			));
		}

		// 开始处理含有顶点关系和距离的字段
		while let Some((lnum, line)) = line_iter.next() {
			let line = line?;
			if line.is_empty() {
				continue;
			}

			// 引号计数
			let mut cnt = 0;
			let mut has_left_quota = false;

			let mut edge: [String; 3] = [String::new(), String::new(), String::new()];
			for char in line.chars() {
				let should_add_edge = has_left_quota;

				if has_left_quota {
					edge[cnt].push(char);
				}

				if char == '\\' {
					has_left_quota = !has_left_quota;
				}

				if char == '"' {
					has_left_quota = !has_left_quota;
				}

				if should_add_edge && !has_left_quota {
					edge[cnt].pop();
					match cnt {
						0 | 1 => {
							g.new_vert(&edge[cnt], false);
						},
						2 => {g.new_edge(
							&edge[0],
							&edge[1],
							edge[2]
								.parse()
								.map_err(|_| error::Error::FileSyntaxWrong(lnum + 1, String::from("distance type wrong")))?,
						)?;},
						// 一行最多三个引号对, 分别对应两个顶点与一个距离
						_ => return Err(error::Error::FileSyntaxWrong(lnum + 1, String::from("too many words"))),
					}
					cnt += 1;
				}
			}
		}
		return Ok(g);
	}
}

use std::{
	char,
	fs::File,
	io::{BufRead, BufReader},
};

use crate::{
	error::{self, Error},
	graph::Graph,
};

struct TextFile {
	file: File,
}

impl TextFile {
	fn open(fpath: &str) -> Result<Self, Error> {
		Ok(TextFile {
			file: File::open(fpath)?,
		})
	}
}

impl TryFrom<TextFile> for Graph {
	type Error = error::Error;

	fn try_from(value: TextFile) -> Result<Self, Self::Error> {
		// let add_to_graph = |g: &mut Graph, id1: String, id2: String, dist: f64| -> error::Result<()> {
		// 	g.new_vert(id1.clone(), false);
		// 	g.new_vert(id2.clone(), false);
		// 	g.new_edge(id1, id2, dist)?;
		// 	Ok(())
		// };

		let mut g = Graph::new();

		let buf = BufReader::new(value.file);
		let mut line_iter = buf.lines().into_iter();

		// 第一行储存的是出口
		// 将它们提前添加到图里
		let exits = line_iter.next().ok_or(error::Error::FileSyntaxWrong)??;
		let mut has_left_quota = false;
		let mut a_exit = String::new();
		for char in exits.chars() {
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
				g.new_vert(a_exit.clone(), true);
				// TODO: 将 graph 中储存的 id 改为 &str 类型
				a_exit.clear();
			}
		}
		if has_left_quota {
			return Err(error::Error::FileSyntaxWrong);
			// TODO: 提示错误出现在哪一行, (可选的)错误详细提示
		}

		// 开始处理含有顶点关系和距离的字段
		while let Some(line) = line_iter.next() {
			let line = line?;
			if line.is_empty() {
				continue;
			}

			// 引号计数
			let mut cnt = 0;
			let mut has_left_quota = false;

			for char in line.chars() {
				let mut edge: [String; 3] = ["".to_string(), "".to_string(), "".to_string()];
				// FIXME: ^
				let should_add_vert = has_left_quota;

				if has_left_quota {
					// 一行最多三个引号对, 分别对应两个顶点与一个距离
					if cnt >= 3 {
						return Err(error::Error::FileSyntaxWrong);
					}
					edge[cnt].push(char);
				}

				if char == '\\' {
					has_left_quota = !has_left_quota;
				}

				if char == '"' {
					has_left_quota = !has_left_quota;
				}

				if should_add_vert && !has_left_quota {
					// TODO: 添加到图里
				}
			}
		}

		todo!()
	}
}

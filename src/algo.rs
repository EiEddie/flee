use std::collections::LinkedList;

use crate::error::*;
use crate::graph::*;

#[derive(Debug, Clone)]
struct Path {
	/// 储存的是顶点和 "与它上一个顶点间的距离" 组成的 tuple.
	points: LinkedList<(*const Vert, f64)>,
}

impl std::fmt::Display for Path {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut iter = self.points.iter();
		write!(f, "[{}]", unsafe { &(*iter.next().unwrap().0).id })?;

		for point in iter {
			write!(f, " -{}-> [{}]", point.1, unsafe { &(*point.0).id })?;
		}

		Ok(())
	}
}

impl Graph {
	#[allow(non_snake_case)]
	fn _DFS_(&self, vert: *mut Vert, dist: f64, path: Path, paths: &mut Vec<Path>) -> Path {
		// 现在对 `vert` 这个顶点进行操作
		// 它与它的上一个顶点间的距离是 `dist`

		let mut this_path = path;

		// 已经陷入环形, 跳过此顶点继续搜索
		if unsafe { (*vert).is_searching } {
			return this_path;
		}

		// 在搜索这个顶点的后继时, 本顶点不可再被进入
		// 这样是为了避免陷入顶点环中
		// Safety: 仅可在单线程操作
		unsafe { (*vert).is_searching = true };

		// 将本顶点放入路径中
		this_path.points.push_back((vert as *const Vert, dist));

		// 当顶点已经是终点(之一)时, 保存这条路径
		// 并继续搜索其余顶点
		if unsafe { (*vert).is_exit } {
			paths.push(this_path.clone());
			this_path.points.pop_back();
			unsafe { (*vert).is_searching = false };
			return this_path;
		}

		// 对后继顶点的搜索
		for Edge { vert, dist } in unsafe { &(*vert).nbrs } {
			this_path = self._DFS_(*vert, *dist, this_path, paths);
		}

		// 本顶点的后继的搜索已经完成
		// 此时本顶点不在路线内, 因为本顶点后面无路可走
		// 在路线内移除本顶点
		this_path.points.pop_back();

		// 本顶点已被搜索完成, 后续的搜索仍可继续进入本顶点
		unsafe { (*vert).is_searching = false };
		return this_path;
	}

	/// 在指定的起点搜索到所有出口的所有路径.
	///
	/// 只要图是联通的, 可以保证至少有一条路, 即 [`Vec`] 内至少有一个元素.
	#[allow(non_snake_case)]
	fn DFS(&mut self, start: String) -> Result<Vec<Path>> {
		let mut paths: Vec<Path> = Vec::new();
		let start: *mut Vert = self.vert_map.get_mut(&start).ok_or(Error::NoVert)?;
		let a_path = Path { points: LinkedList::new(), };

		self._DFS_(start, 0., a_path, &mut paths);

		return Ok(paths);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn find_path() {
		// [1] <-1.2-> [2] <-2.3-> [*3]
		// [4] <-1.4-> [1]
		// [4] <-2.4-> [2]
		// [4] <-3.4-> [*3]
		let mut g = Graph::new();
		g.new_vert(String::from("1"), false);
		g.new_vert(String::from("2"), false);
		g.new_vert(String::from("3"), true);
		g.new_vert(String::from("4"), false);
		let _ = g.new_edge(String::from("1"), String::from("2"), 1.2);
		let _ = g.new_edge(String::from("2"), String::from("3"), 2.3);
		let _ = g.new_edge(String::from("1"), String::from("4"), 1.4);
		let _ = g.new_edge(String::from("2"), String::from("4"), 2.4);
		let _ = g.new_edge(String::from("3"), String::from("4"), 3.4);

		for (index, path) in g.DFS(String::from("1")).unwrap().iter().enumerate() {
			println!("{}: {path}", index + 1);
		}
	}
}

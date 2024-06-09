use std::collections::{HashSet, LinkedList};

use crate::error::*;
use crate::graph::*;

#[derive(Debug, Clone)]
struct Path<'a> {
	/// 储存的是顶点和 "与它上一个顶点间的距离" 组成的 tuple.
	points: LinkedList<(*const Vert<'a>, f64)>,
}

impl<'a> std::fmt::Display for Path<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut iter = self.points.iter();
		write!(f, "[{}]", unsafe { &(*iter.next().unwrap().0).id })?;

		for point in iter {
			write!(f, " -{}-> [{}]", point.1, unsafe { &(*point.0).id })?;
		}

		Ok(())
	}
}

impl<'a> Graph<'a> {
	/// 仅供 `DFS` 方法使用的内部函数, 用于递归地搜索所有路径
	///
	/// # Parameters
	///
	/// - `vert_ptr`: 指向函数当前操作顶点的指针
	/// - `dist`: 当前顶点与它的父顶点之间的距离
	/// - `passed_vert`: 在这次搜索过程中已经走过的顶点, 动态更新
	/// - `path`: 用于储存正在搜索的路径中本顶点前所有顶点
	/// - `paths`: 用于储存搜索到的所有路径
	#[allow(non_snake_case)]
	fn _DFS_(&self, vert_ptr: *const Vert<'a>, dist: f64,
	         passed_vert: &mut HashSet<*const Vert<'a>>, path: Path<'a>,
	         paths: &mut Vec<Path<'a>>)
	         -> Path<'a> {
		// 现在对 `vert` 这个顶点进行操作
		// 它与它的上一个顶点间的距离是 `dist`
		let vert: &Vert<'a> = unsafe { &*vert_ptr };

		let mut this_path = path;

		// 已经陷入环形, 跳过此顶点继续搜索
		// 在搜索这个顶点的后继时, 本顶点不可再被进入
		// 这样是为了避免陷入顶点环中
		if !passed_vert.insert(vert_ptr) {
			return this_path;
		}

		// 将本顶点放入路径中
		this_path.points.push_back((vert_ptr, dist));

		// 当顶点已经是终点(之一)时, 保存这条路径
		if vert.is_exit {
			paths.push(this_path.clone());
		} else {
			// 对后继顶点的搜索
			for Edge { vert, dist } in &vert.nbrs {
				this_path = self._DFS_(*vert, *dist, passed_vert, this_path, paths);
			}
		}

		// 本顶点的后继的搜索已经完成
		// 此时本顶点不在路线内, 因为本顶点后面无路可走
		// 在路线内移除本顶点
		this_path.points.pop_back();

		// 本顶点已被搜索完成, 后续的搜索仍可继续进入本顶点
		passed_vert.remove(&vert_ptr);
		return this_path;
	}

	/// 在指定的起点搜索到所有出口的所有路径.
	///
	/// 只要图是联通的, 可以保证至少有一条路, 即 [`Vec`] 内至少有一个元素.
	#[allow(non_snake_case)]
	pub fn DFS(&self, start: &String) -> Result<Vec<Path<'a>>> {
		let mut paths: Vec<Path> = Vec::new();
		let start = self.get(start).ok_or(Error::NoVert)? as *const Vert<'a>;
		let mut passed_vert = HashSet::new();
		let a_path = Path { points: LinkedList::new(), };

		self._DFS_(start, 0., &mut passed_vert, a_path, &mut paths);

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
		g.new_vert(&String::from("1"), false);
		g.new_vert(&String::from("2"), false);
		g.new_vert(&String::from("3"), true);
		g.new_vert(&String::from("4"), false);
		g.new_edge(&String::from("1"), &String::from("2"), 1.2)
		 .unwrap();
		g.new_edge(&String::from("2"), &String::from("3"), 2.3)
		 .unwrap();
		g.new_edge(&String::from("1"), &String::from("4"), 1.4)
		 .unwrap();
		g.new_edge(&String::from("2"), &String::from("4"), 2.4)
		 .unwrap();
		g.new_edge(&String::from("3"), &String::from("4"), 3.4)
		 .unwrap();

		for (index, path) in g.DFS(&String::from("1")).unwrap().iter().enumerate() {
			println!("{}: {path}", index + 1);
		}
	}
}

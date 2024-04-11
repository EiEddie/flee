use std::collections::LinkedList;

use crate::error::*;
use crate::graph::*;

#[derive(Debug, Clone)]
struct Path {
	/// 储存的是顶点和 "与它上一个顶点间的距离" 组成的 tuple.
	point: LinkedList<(*const Vert, f64)>,
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
		this_path.point.push_back((vert as *const Vert, dist));

		// 当顶点已经是终点(之一)时, 保存这条路径
		// 并继续搜索其余顶点
		if unsafe { (*vert).is_exit } {
			paths.push(this_path.clone());
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
		this_path.point.pop_back();

		// 本顶点已被搜索完成, 后续的搜索仍可继续进入本顶点
		unsafe { (*vert).is_searching = false };
		return this_path;
	}

	#[allow(non_snake_case)]
	fn DFS(&mut self, start: String) -> Result<Vec<Path>> {
		let mut paths: Vec<Path> = Vec::new();
		let start: *mut Vert = self.vert_map.get_mut(&start).ok_or(Error::NoVert)?;
		let a_path = Path { point: LinkedList::new(), };

		self._DFS_(start, 0., a_path, &mut paths);

		return Ok(paths);
	}
}

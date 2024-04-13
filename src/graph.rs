use std::collections::{HashMap, LinkedList};

use crate::error::*;

pub(crate) struct Edge<'a> {
	// NOTE: 保证通过指针仅能改变 `is_search` 的值.
	pub(crate) vert: *mut Vert<'a>,
	pub(crate) dist: f64,
}

pub(crate) struct Vert<'a> {
	/// 顶点的唯一标识名.
	pub(crate) id: &'a String,

	/// 是否是出口.
	pub(crate) is_exit: bool,

	/// 节点的邻点列表.
	/// 储存的是 [`Edge`]
	pub(crate) nbrs: Vec<Edge<'a>>,

	/// 标识是否正在遍历
	pub(crate) is_searching: bool,
}

pub(crate) struct Graph<'a> {
	/// # Waring
	///
	/// !!! 必须保证指向此容器内的任意元素的引用在结构体被丢弃之前始终有效 !!!
	///
	/// 也就是说不允许任何删除此容器内元素的行为, 也不允许任何种类的内存再分配.
	/// 因此此容器使用 [`LinkedList`], 因为诸如 [`Vec`] 会进行内存扩容与值在内存中的移动.
	ids: LinkedList<String>,

	/// 建立顶点与实际平面图间的对应关系.
	///
	/// - `key`: 顶点的 id
	/// - `value`: 顶点, 拥有其所有权
	pub(crate) vert_map: HashMap<&'a String, Vert<'a>>,
}

impl<'a> Graph<'a> {
	pub(crate) fn new() -> Self {
		Graph { ids:      LinkedList::new(),
		        vert_map: HashMap::new(), }
	}

	/// 添加一个新的顶点, `id` 与 `is_exit` 字段由参数指定.
	///
	/// # Returns
	/// 若已存在 id 重复的顶点, 返回 `true`; 否则返回 `false`.
	pub(crate) fn new_vert(&mut self, id: &String, is_exit: bool) -> bool {
		let is_exist = self.vert_map.contains_key(&id);
		if !is_exist {
			self.ids.push_back(id.clone());
			// Safety:
			// 能保证 `self.ids` 只进不出
			// 也就是说指向这个容器内元素的引用在结构体生命周期内始终有效
			let id: &'a String = unsafe {
				let raw_ptr = self.ids.back().unwrap() as *const String;
				&*raw_ptr
			};
			let v = Vert { id,
			               is_exit,
			               nbrs: Vec::new(),
			               is_searching: false };
			self.vert_map.insert(&v.id, v);
		}
		return is_exist;
	}

	/// 添加一条单向的边, 从 `from` 指向 `to`, 长度为 `dist`.
	///
	/// # Returns
	/// 当给定的 `id` 不存在时, 给出一个 `NoVert` 错误
	fn _new_edge_forward_(&mut self, from: &String, to: &String, dist: f64) -> Result<()> {
		let to: *mut Vert = self.vert_map.get_mut(to).ok_or(Error::NoVert)?;
		self.vert_map
		    .get_mut(from)
		    .ok_or(Error::NoVert)?
		    .nbrs
		    .push(Edge { vert: to, dist });
		Ok(())
	}

	/// 添加一条双向的边, 长度为 `dist`.
	///
	/// # Returns
	/// 当给定的 `id` 不存在时, 给出一个 `NoVert` 错误
	pub(crate) fn new_edge(&mut self, v1: &String, v2: &String, dist: f64) -> Result<()> {
		self._new_edge_forward_(v1, v2, dist)?;
		self._new_edge_forward_(v2, v1, dist)?;
		Ok(())
	}
}

impl<'a> std::fmt::Display for Graph<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for (id, vert) in &self.vert_map {
			for edge in &vert.nbrs {
				// Safety: read only
				let id2 = unsafe { &(*edge.vert).id };
				writeln!(
				         f,
				         "[{}{}] <-{}-> [{}{}]",
				         if vert.is_exit { "*" } else { "" },
				         id,
				         edge.dist,
				         if unsafe { (*edge.vert).is_exit } {
					         "*"
				         } else {
					         ""
				         },
				         id2
				)?;
			}
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn print_graph() {
		// [1] <-0.5-> [2] <-1.2-> [3]
		let mut g = Graph::new();
		g.new_vert(&String::from("1"), false);
		g.new_vert(&String::from("2"), false);
		g.new_vert(&String::from("3"), false);
		let _ = g.new_edge(&String::from("1"), &String::from("2"), 0.5);
		let _ = g.new_edge(&String::from("3"), &String::from("2"), 1.2);

		println!("{g}");
	}
}

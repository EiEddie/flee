use std::collections::{HashMap, HashSet, LinkedList};
use std::hash::Hash;

use crate::error::*;

#[derive(Debug)]
pub(crate) struct Edge<'a> {
	// NOTE: 保证通过指针仅能改变 `is_search` 的值.
	pub(crate) vert: *mut Vert<'a>,
	pub(crate) dist: f64,
}

impl<'a> Hash for Edge<'a> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		// 当 vert 的 id 相同时, 两者 vert 指针指向的位置始终相同
		// 因为 vert 保存在一个 LinkedList  内
		// 而 Edge 的 vert 指针始终指向 LinkedList 中的对应值
		std::ptr::hash(self.vert, state);
	}
}

impl<'a> PartialEq for Edge<'a> {
	fn eq(&self, other: &Self) -> bool {
		// 为了保证当两者哈希值相同时, 两者始终相等
		unsafe { (*self.vert).id == (*other.vert).id }
	}
}

impl<'a> Eq for Edge<'a> {}

#[derive(Debug)]
pub(crate) struct Vert<'a> {
	/// 顶点的唯一标识名.
	pub(crate) id: &'a String,

	/// 是否是出口.
	pub(crate) is_exit: bool,

	/// 节点的邻点列表.
	/// 储存的是 [`Edge`]
	pub(crate) nbrs: HashSet<Edge<'a>>,

	/// 标识是否正在遍历
	pub(crate) is_searching: bool,
}

#[derive(Debug)]
pub(crate) struct Graph<'a> {
	/// 存储每个顶点 `id` 的容器, 拥有所有 `id` 的所有权
	///
	/// # Waring
	///
	/// !!! 必须保证指向此容器内的任意元素的引用在结构体被丢弃之前始终有效 !!!
	///
	/// 也就是说不允许任何删除此容器内元素的行为, 也不允许任何种类的内存再分配.
	/// 因此此容器使用 [`LinkedList`], 因为诸如 [`Vec`] 会进行内存扩容与值在内存中的移动.
	ids: LinkedList<String>,

	/// 存储每个顶点的容器, 拥有所有顶点的所有权
	///
	/// # Waring
	///
	/// !!! 必须保证指向此容器内的任意元素的指针在结构体被丢弃之前始终有效 !!!
	///
	/// 也就是说不允许任何删除此容器内元素的行为, 也不允许任何种类的内存再分配.
	/// 因此此容器使用 [`LinkedList`], 因为诸如 [`Vec`] 会进行内存扩容与值在内存中的移动.
	verts: LinkedList<Vert<'a>>,

	/// 建立顶点与实际平面图间的对应关系.
	///
	/// - `key`: 顶点的 id
	/// - `value`: 顶点
	vert_map: HashMap<&'a String, *mut Vert<'a>>,
}

impl<'a> Graph<'a> {
	pub(crate) fn new() -> Self {
		Graph { ids:      LinkedList::new(),
		        verts:    LinkedList::new(),
		        vert_map: HashMap::new(), }
	}

	pub(crate) fn get(&self, id: &String) -> Option<&Vert<'a>> {
		// Safety:
		// 能保证 `self.verts` 只进不出
		// 也就是说指向这个容器内元素的指针在结构体生命周期内始终有效
		self.vert_map.get(id).map(|v| unsafe { &**v })
	}

	pub(crate) fn get_mut(&mut self, id: &String) -> Option<&mut Vert<'a>> {
		// Safety:
		// 能保证 `self.verts` 只进不出
		// 也就是说指向这个容器内元素的指针在结构体生命周期内始终有效
		self.vert_map.get(id).map(|v| unsafe { &mut **v })
	}

	/// 添加一个新的顶点, `id` 与 `is_exit` 字段由参数指定.
	///
	/// # Returns
	/// 若已存在 id 重复的顶点, 返回 `true`; 否则返回 `false`.
	pub(crate) fn new_vert(&mut self, id: &String, is_exit: bool) -> bool {
		let is_exist = self.vert_map.contains_key(&id);

		// 如果给定的 id 不存在
		// 即对应的顶点不存在
		if !is_exist {
			// 将 id 放入 id 池内, 并获得这个 id 的引用
			self.ids.push_back(id.clone());
			// Safety:
			// 能保证 `self.ids` 只进不出
			// 也就是说指向这个容器内元素的引用在结构体生命周期内始终有效
			let id: &'a String = unsafe {
				let raw_ptr = self.ids.back().unwrap() as *const String;
				&*raw_ptr
			};

			// 将新建的顶点放入顶点池内, 并获得对这个顶点的可变引用
			let v = Vert { id,
			               is_exit,
			               nbrs: HashSet::new(),
			               is_searching: false };
			// 在顶点的 id 的借用的所有权被转移之前复制一份借用
			let id = v.id;
			self.verts.push_back(v);
			let v = self.verts.back_mut().unwrap() as *mut Vert;

			// 将顶点信息放入对应表内
			self.vert_map.insert(id, v);
		}
		return is_exist;
	}

	/// 添加一条单向的边, 从 `from` 指向 `to`, 长度为 `dist`.
	///
	/// # Returns
	///
	/// - 当给定的 `id` 不存在时, 给出一个 `NoVert` 错误
	/// - 当这条边指向它自身时, 给出一个 `SelfEdge` 错误
	/// - 当已经有一条指向 `to` 的边时, 给出一个 `DoubleEdge` 错误
	fn _new_edge_forward_(&mut self, from: &String, to: &String, dist: f64) -> Result<()> {
		if from == to {
			return Err(Error::SelfEdge);
		}
		let to: *mut Vert = self.get_mut(to).ok_or(Error::NoVert)?;
		if !self.get_mut(from)
		        .ok_or(Error::NoVert)?
		        .nbrs
		        .insert(Edge { vert: to, dist })
		{
			return Err(Error::DoubleEdge);
		}
		Ok(())
	}

	/// 添加一条双向的边, 长度为 `dist`.
	///
	/// # Returns
	///
	/// - 当给定的 `id` 不存在时, 给出一个 `NoVert` 错误
	/// - 当这条边指向它自身时, 给出一个 `SelfEdge` 错误
	/// - 当已经有一条指向 `to` 的边时, 给出一个 `DoubleEdge` 错误
	pub(crate) fn new_edge(&mut self, v1: &String, v2: &String, dist: f64) -> Result<()> {
		self._new_edge_forward_(v1, v2, dist)?;
		self._new_edge_forward_(v2, v1, dist)?;
		Ok(())
	}
}

impl<'a> std::fmt::Display for Graph<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for (&id, &vert) in &self.vert_map {
			for edge in unsafe { &(*vert).nbrs } {
				// Safety: read only
				let id2 = unsafe { (*edge.vert).id };
				writeln!(
				         f,
				         "[{}{}] <-{}-> [{}{}]",
				         if unsafe { (*vert).is_exit } { "*" } else { "" },
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
		g.new_edge(&String::from("1"), &String::from("2"), 0.5)
		 .unwrap();
		// g.new_edge(&String::from("1"), &String::from("2"), 2.3).unwrap();
		g.new_edge(&String::from("3"), &String::from("2"), 1.2)
		 .unwrap();

		println!("{g}");
	}
}

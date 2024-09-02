use std::collections::{HashMap, HashSet, LinkedList};
use std::hash::Hash;

use crate::error::*;

#[derive(Debug)]
pub(crate) struct Edge<'a> {
	pub(crate) vert: *const Vert<'a>,
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
}

#[derive(Debug)]
pub struct Graph<'a> {
	/// 节点 id 池
	///
	/// 储存所有的 id, 用于 Drop 时释放内存
	///
	/// ! 禁止删除或更改内部的任意元素
	id_poll: LinkedList<*mut String>,

	/// 建立顶点与实际平面图间的对应关系.
	///
	/// - `key`: 顶点的 id
	/// - `value`: 顶点
	vert_map: HashMap<&'a String, *mut Vert<'a>>,
}

impl<'a> Graph<'a> {
	pub(crate) fn new() -> Self {
		Graph { id_poll:  LinkedList::new(),
		        vert_map: HashMap::new(), }
	}

	pub(crate) fn get(&self, id: &String) -> Option<&Vert<'a>> {
		// Safety:
		// 指针在结构体 Drop 之前始终有效
		self.vert_map.get(id).map(|v| unsafe { &**v })
	}

	pub(crate) fn get_mut(&mut self, id: &String) -> Option<&mut Vert<'a>> {
		// Safety:
		// 指针在结构体 Drop 之前始终有效
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
			// 将 id 存放在堆上
			let id = Box::leak(Box::new(id.clone()));
			// 将 id 的可变引用存放在 id 池中
			// 在结构体 Drop 时将释放池中所有元素
			self.id_poll.push_back(id);
			// 将新建的顶点放在堆上, 并获得对这个顶点的可变引用
			let v = Box::leak(Box::new(Vert { id,
			                                  is_exit,
			                                  nbrs: HashSet::new() }));
			// 将顶点信息放入对应表内
			// 只使用 id 的不可变借用
			self.vert_map.insert(id as &String, v);
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

impl<'a> Drop for Graph<'a> {
	fn drop(&mut self) {
		for id in self.id_poll.iter() {
			drop(unsafe { Box::from_raw(*id) });
		}
		for (_, v) in self.vert_map.iter() {
			drop(unsafe { Box::from_raw(*v) });
		}
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
		g.new_edge(&String::from("3"), &String::from("2"), 1.2)
		 .unwrap();

		println!("{g}");
	}

	#[test]
	fn mem_safe() {
		let mut g = Graph::new();
		for i in 0..10000 {
			g.new_vert(&i.to_string(), false);
		}
		for j in 1..10000 {
			g.new_edge(&String::from("0"), &j.to_string(), j as f64)
			 .unwrap();
		}

		for k in &g.get(&String::from("0")).unwrap().nbrs {
			let v = unsafe { &*k.vert };
			assert!(!v.is_exit);
		}
	}

	#[test]
	fn edge_error() {
		let mut g = Graph::new();

		if let Err(Error::NoVert) = g.new_edge(&String::from("1"), &String::from("2"), 0.) {
		} else {
			panic!("NoVert err is not triggered");
		}

		g.new_vert(&String::from("1"), false);
		g.new_vert(&String::from("2"), false);
		if let Err(Error::SelfEdge) = g.new_edge(&String::from("1"), &String::from("1"), 0.) {
		} else {
			panic!("SelfEdge err is not triggered");
		}

		g.new_edge(&String::from("1"), &String::from("2"), 1.)
		 .unwrap();
		if let Err(Error::DoubleEdge) = g.new_edge(&String::from("2"), &String::from("1"), 2.) {
		} else {
			panic!("DoubleEdge err is not triggered");
		}
	}
}

use std::ptr::NonNull;

use aporia_core::geometry::Size;

use crate::{geometry::Constraint, node::LayoutNode, offset::compute_offset, size::compute_size};

// レイアウトデータの実体を保持するための構造体
pub struct LayoutTree {
	chunk_allocator: ChunkedNodeAllocator,
	top: NonNull<LayoutNode>,
}

impl LayoutTree {
	pub fn new() -> Self {
		let chunk_allocator = ChunkedNodeAllocator::new();
		let parent_node: *mut LayoutNode = chunk_allocator.alloc();

		Self { chunk_allocator, top: unsafe { NonNull::new_unchecked(parent_node) } }
	}

	pub fn compute(&mut self, constraint: Constraint) {
		// 最上位の要素はpreferred size以外がプレーンなBoxとして事前に定義されていることが約束される
		// そのBoxのpreferred sizeを直接設定することでウィンドウサイズを伝達する
		let parent_node = unsafe { self.top.as_mut() };
		parent_node.set_size(constraint.max_width, constraint.max_height);
		compute_size(parent_node, constraint);
		compute_offset(parent_node, Size::new(0f32, 0f32));
	}
}

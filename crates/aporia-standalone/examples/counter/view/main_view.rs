use aporia_standalone::{
	component::{Render, View},
	reactivity::Signal,
};
use aporia_std::Block;

use crate::{AppContext, AppState};

pub(crate) struct MainView {
	count: Signal<i32>,
}

impl MainView {
	pub fn new(count: Signal<i32>) -> Self {
		Self { count }
	}
}

impl Render<AppState> for MainView {
	fn view(&self, cx: &mut AppContext) -> View {
		if cx.read(self.count) == 2 {
			cx.render(Block::default())
		} else {
			cx.render(Block::default().child(Block::default()))
		}
	}
}

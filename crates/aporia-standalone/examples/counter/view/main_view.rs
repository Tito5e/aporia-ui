use aporia_standalone::{
	component::{Component, View, ViewCx},
	reactivity::Signal,
};

use crate::{AppContext, AppState};

pub(crate) struct MainView {
	count: Signal<i32>,
}

impl MainView {
	pub fn new(count: Signal<i32>) -> Self {
		Self { count }
	}
}

impl Component<AppState> for MainView {
	fn view(&self, cx: &mut AppContext) -> View {
		if cx.read(self.count) == 2 {
			cx.render(Block::new())
		} else {
			cx.render(Block::new().child(Block::new()))
		}
	}
}

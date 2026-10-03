use aporia_ui::{
	component::{Component, View, ViewCx},
	reactivity::Signal,
	widget::Block,
};

pub(crate) struct MainView {
	count: Signal<i32>,
}

impl MainView {
	pub fn new(count: Signal<i32>) -> Self {
		Self { count }
	}
}

impl Component for MainView {
	fn view(&self, cx: &mut ViewCx) -> View {
		if cx.read(self.count) == 2 {
			cx.render(Block::new())
		} else {
			cx.render(Block::new().child(Block::new()))
		}
	}
}

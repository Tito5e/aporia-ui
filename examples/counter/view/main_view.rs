use aporia_ui::{
	component::{Component, View},
	reactivity::signal::Signal,
	widget::Block,
};

pub(crate) struct MainView {
	count: Signal<i32>,
}

impl MainView {
	pub fn new() -> Self {
		Self { count: Signal::new(0) }
	}
}

impl Component for MainView {
	fn view(&self) -> View {
		if self.count.get() == &2 {
			Block::new().into()
		} else {
			Block::new().child(Block::new()).into()
		}
	}
}

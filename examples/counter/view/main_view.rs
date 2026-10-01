use aporia_ui::{
	component::{Builder, Component, Finalized, Finalizer},
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
	fn view(self, finalizer: Finalizer<Self>) -> Finalized {
		if self.count.get() == &2 {
			finalizer.finalize(self, Block::new())
		} else {
			finalizer.finalize(self, Block::new().child(Block::new()))
		}
	}
}

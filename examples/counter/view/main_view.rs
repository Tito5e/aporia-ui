use aporia_ui::{
	component::{Component, Render},
	reactivity::signal::Signal,
	widget::Block,
};

pub(crate) struct MainView {}

impl MainView {
	pub fn new() -> Self {
		Self {}
	}
}

impl Component for MainView {
	fn view(self) -> impl Render + 'static {
		let count = Signal::new(0);

		move || Block::new()
	}
}

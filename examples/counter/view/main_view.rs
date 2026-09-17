use aporia_ui::{
	component::{Builder, Component},
	widget::Block,
};

pub(crate) struct MainView {}

impl MainView {
	pub fn new() -> Self {
		Self {}
	}
}

impl Component for MainView {
	fn view(&self) -> impl Builder {
		//let count = Signal::new(0);

		Block::new()
	}
}

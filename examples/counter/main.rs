use crate::view::main_view::MainView;
use aporia_ui::{standalone::StandaloneApplication, window::Window};

mod view;

pub fn main() {
	let application = StandaloneApplication::new();
	let _result = application.run(|cx| {
		cx.create_window(Window::mount(MainView::new()).with_title("Counter"));
	});
}

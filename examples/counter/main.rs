use crate::view::main_view::MainView;
use aporia_ui::{standalone::StandaloneApplication, window::Window};
use env_logger::Env;

mod view;

pub fn main() {
	env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

	let application = StandaloneApplication::new();
	let _result = application.run(|cx| {
		cx.create_window(Window::mount(MainView::new()).with_title("Counter"));
	});
}

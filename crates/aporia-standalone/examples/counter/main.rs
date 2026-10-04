use crate::view::main_view::MainView;
use aporia_ui::{
	component::ViewCx, reactivity::Signal, standalone::StandaloneApplication, window::Window,
};
use env_logger::Env;

mod view;

pub struct AppState {
	count: Signal<i32>,
}
pub type AppContext = ViewCx<AppState>;

pub fn main() {
	env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

	let application = StandaloneApplication::with_state(|cx| {
		let count = cx.global_signal(0);

		AppState { count }
	});
	let _ = application.run(|cx| {
		cx.create_window(Window::mount(MainView::new(count)).with_title("Counter"));
	});
}

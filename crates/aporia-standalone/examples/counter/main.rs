use crate::view::main_view::MainView;
use aporia_standalone::{
	application::StandaloneApplication,
	component::{Component, ViewCx},
	reactivity::Signal,
	window::Window,
};
use env_logger::Env;

mod view;

pub struct AppState {
	count: Signal<i32>,
}
pub type AppContext<'a, 'b> = ViewCx<'a, 'b, AppState>;

pub fn main() {
	env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

	let application = StandaloneApplication::with_state(|cx| {
		let count = cx.global_signal(0);

		AppState { count }
	});
	let _ = application.run(|cx| {
		let count = cx.global_state().count;
		let main_view = MainView::new(count);
		cx.create_window(Window::mount(Component::new(main_view)).with_title("Counter"));
	});
}

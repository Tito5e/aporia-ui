use crate::widget::Widget;

pub trait Builder {
	#[doc(hidden)]
	fn build(self) -> Box<dyn Widget>;
}

pub struct NoChild;

impl Builder for NoChild {
	fn build(self) -> Box<dyn Widget> {
		unreachable!("Called NoChild::build")
	}
}

pub trait Component {
	fn view(&self) -> impl Builder;
}

pub struct ComponentBuilder<C: Component> {
	pub(crate) component: C,
}

impl<C: Component> ComponentBuilder<C> {
	pub fn new(component: C) -> Self {
		Self { component }
	}
}

impl<C: Component> Builder for ComponentBuilder<C> {
	fn build(self) -> Box<dyn Widget> {
		todo!("ここにScopeの確保コードやコンポーネントの実体化コードを書く")
	}
}

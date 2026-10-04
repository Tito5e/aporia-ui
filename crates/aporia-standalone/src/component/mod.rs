mod builder;
mod context;
mod state;

pub use builder::Builder;
pub use context::ViewCx;
pub(crate) use state::ComponentState;

use crate::{application::StandaloneCx, widget::WidgetHandle};

pub struct View(WidgetHandle);

pub trait Component {
	fn view<S>(&self, cx: &mut ViewCx<S>) -> View;
}

impl<C: Component + 'static> Builder for C {
	fn build(self, cx: &mut StandaloneCx) -> WidgetHandle {
		// Reserve heap storage for the component.
		// Because the component's Reconciler must point back to the component itself, the pointer address must be determined in advance.
		//
		// Therefore, use WidgetHandle::reserve instead of the regular WidgetHandle::new
		let reservation = WidgetHandle::reserve::<ComponentState<C>>();

		// The component needs an independent Scope to own the Signal defined in its view function and serve as the parent scope for its own Reconciler effect.
		let mut scope = cx.create_scope();

		// The component has a Reconciler effect responsible for triggering its own re-evaluation.
		let mut reconciler = cx.reconcile::<ComponentState<C>>(&mut scope, reservation.as_ptr());
		let mut view_cx = ViewCx {
			ctx: cx,
			scope: &mut scope,
			reconciler: &mut reconciler,
			global_state: &mut cx.global_state,
		};
		let child = self.view(&mut view_cx);

		reservation.write(ComponentState { component: self, scope, child: child.0, reconciler })
	}
}

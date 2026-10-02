use crate::reactivity::{Read, Subscribe, effect::EffectHeader};

#[derive(Clone, Copy)]
pub struct Mapped<P, F> {
	parents: P,
	f: F,
}

impl<P: Read, R, F: for<'a> Fn(P::Out<'a>) -> R> Mapped<P, F> {
	pub fn new(parents: P, f: F) -> Self {
		Self { parents, f }
	}

	pub fn get(&self) -> R {
		(self.f)(self.parents.read())
	}
}

impl<P: Read, R, F: for<'a> Fn(P::Out<'a>) -> R> Read for Mapped<P, F> {
	type Out<'a>
		= R
	where
		Self: 'a;

	fn read(&self) -> R {
		self.get()
	}
}

impl<P: Subscribe, F> Subscribe for Mapped<P, F> {
	fn subscribe(&self, effect: &mut EffectHeader) {
		self.parents.subscribe(effect);
	}
}

macro_rules! impl_tuple {
    ($($n:ident : $i:tt),+) => {
        impl<$($n: Subscribe),+> Subscribe for ($($n,)+) {
            fn subscribe(&self, effect: &mut EffectHeader) { $( self.$i.subscribe(effect); )+ }
        }
        impl<$($n: Read),+> Read for ($($n,)+) {
            type Out<'a> = ($($n::Out<'a>,)+) where Self: 'a;
            fn read(&self) -> Self::Out<'_> { ($( self.$i.read(), )+) }
        }
    };
}
impl_tuple!(A:0);
impl_tuple!(A:0, B:1);
impl_tuple!(A:0, B:1, C:2);
impl_tuple!(A:0, B:1, C:2, D:3);

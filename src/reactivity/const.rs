use crate::reactivity::{Source, effect::EffectHeader};

#[derive(Clone, Copy)]
pub struct Const<T>(T);

impl<T> Const<T> {
	pub fn new(value: T) -> Self {
		Self(value)
	}
}

impl<T: Clone> Source for Const<T> {
	type Out = T;

	fn read(&self) -> T {
		self.0.clone()
	}

	fn subscribe(&self, _: &mut EffectHeader) -> T {
		self.0.clone()
	}
}

macro_rules! impl_tuple {
    ($($n:ident : $i:tt),+) => {
        impl<$($n: Source),+> Source for ($($n,)+) {
            type Out = ($($n::Out,)+);
            fn read(&self) -> Self::Out {
				($( self.$i.read(), )+)
			}

            fn subscribe(&self, effect: &mut EffectHeader) -> Self::Out {
				($( self.$i.subscribe(effect), )+)
			}
        }
    };
}
impl_tuple!(A:0);
impl_tuple!(A:0, B:1);
impl_tuple!(A:0, B:1, C:2);
impl_tuple!(A:0, B:1, C:2, D:3);

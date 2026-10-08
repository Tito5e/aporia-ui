use std::collections::HashMap;

use crate::reactivity::{Source, effect::EffectHeader};

#[derive(Clone, Copy)]
pub struct Const<T>(T);

impl<T> Const<T> {
	pub fn new(value: T) -> Self {
		Self(value)
	}
}

pub struct Test {}

impl Test {
	fn get_or_insert(map: &mut HashMap<u32, String>, k: u32) -> &String {
		if let Some(v) = map.get(&k) {
			return v; // 戻り値として借用が関数外に出る
		}
		map.insert(k, String::new()); // NLLはここを拒否する
		map.get(&k).unwrap()
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

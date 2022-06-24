#![allow(unused_imports, dead_code)]
pub(crate) mod apu;
mod bus;
mod ctrl;
mod nes;
pub(crate) mod ppu;
mod rom;
mod six502;
mod io;
pub(crate) mod mapper;


macro_rules! impl_deref_mut {
	($($struct_name:ident {$field:ident}),+ $(,)?) => {
		$(
			impl Deref for $struct_name {
				type Target = u8;
				fn deref(&self) -> &Self::Target {
					&self.$field
				}
			}

			impl DerefMut for $struct_name {
				fn deref_mut(&mut self) -> &mut Self::Target {
			        &mut self.$field
			    }
			}
		)+
	};
}

pub(crate) use impl_deref_mut;

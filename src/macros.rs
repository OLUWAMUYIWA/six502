macro_rules! impl_deref_mut {
	($($struct_name:ident {$field:ident}),+ $(,)?) => {
		$(
			impl Deref for $struct_name {
				type Target = [u8];
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

	($($sruct_name:ident ($field:ident)),+, + $(,)?) => {
		$(
			impl Deref for $struct_name {
				type Target = u8;
				fn deref(&self) -> &Self::Target {
					&self.$field
				}
			}

			impl DerefMut for $sruct_name {
				fn deref_mut(&mut self) -> &mut Self::Target {
					&mut &self.$field
				}
			}
		)+
	}
}

pub(crate) use impl_deref_mut;
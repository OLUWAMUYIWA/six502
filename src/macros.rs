macro_rules! impl_deref_mut {
	// for structs with [u8; x] fields
	($($struct_name:ident {$field:ident, $type:ty}),+ $(,)?) => {
		$(
			impl Deref for $struct_name {
				type Target = $type;
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
	// for structs with `u8` fields
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

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
	($($struct_name:ident ($field:ident)),+, + $(,)?) => {
		$(
			impl Deref for $struct_name {
				type Target = u8;
				fn deref(&self) -> &Self::Target {
					&self.$field
				}
			}

			impl DerefMut for $struct_name {
				fn deref_mut(&mut self) -> &mut Self::Target {
					&mut &self.$field
				}
			}
		)+
	}
}
// Group 1: ADC, AND, CMP, EOR, LDA, ORA, SBC, STA
macro_rules! impl_addr_modes {
	($($t:ty),+ ; 1) => {
		$(
			impl<A: crate::six502::addr_mode::AcceptableAddrModes> crate::six502::addr_mode::AcceptableAddrModes for $t {
				const ACCUMULATOR: bool = false;

				const ABSOLUTE: bool = true ;

				const ABS_X_IDXD: bool = true;

				const ABS_Y_IDXD: bool = true;

				const IMMEDIATE: bool = true;

				const INDIRECT: bool = false;

				const XIDXD_INDIRECT: bool = true;

				const INDIRECT_Y_IDXD: bool = true;

				const ZP: bool = true;

				const ZP_X_IDXD: bool = true;

				const ZP_Y_IDXD: bool = false;

				const IMPLIED: bool = false;

				const RELATIVE: bool = false;
		})+
	};
}

pub(crate) use impl_addr_modes;
pub(crate) use impl_deref_mut;

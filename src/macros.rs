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
			impl<A: crate::six502::addr_mode::AcceptableAddrModes6502> crate::six502::addr_mode::AcceptableAddrModes6502 for $t {
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

pub(crate) trait AcceptableAddrModes6502 {
    // OPC means `opcode`.
    // operand is the accumulator. for single byte instructions
    const ACCUMULATOR: bool;

    // OPC $LLHH: operand is address $HHLL (i.e. read little-endian)
    const ABSOLUTE: bool;

    // Next two are Absolute Indexed.
    // Absolute indexed address is absolute addressing with an index register added to the absolute address.

    // OPC $LLHH,X: operand is address; effective address is address incremented by X with carry
    const ABS_X_IDXD: bool;

    // OPC $LLHH,Y: operand is address; effective address is address incremented by Y with carry
    const ABS_Y_IDXD: bool;

    // OPC #$BB: operand is the byte BB, as is.
    const IMMEDIATE: bool;

    const INDIRECT: bool;

    // OPC ($LLHH): operand is address; effective address is contents of word at address: C.w($HHLL)
    // Indirect, // Indirect was excluded because it yields a u16 value and is only useful in the `jmpi` instruction

    // operand is zeropage address; effective address is word in (LL + X, LL + X + 1), inc. without carry: C.w($00LL + X)
    const XIDXD_INDIRECT: bool;

    // operand is zeropage address; effective address is word in (LL, LL + 1) incremented by Y with carry: C.w($00LL) + Y
    const INDIRECT_Y_IDXD: bool;
    //Relative

    //This type of addressing is called “zero page” - only the first page (the first 256 bytes) of memory is accessible
    const ZP: bool;

    // OPC $LL,X    operand is zeropage address; effective address is address incremented by X without carry
    const ZP_X_IDXD: bool;

    // OPC $LL,Y    operand is zeropage address; effective address is address incremented by Y without carry
    const ZP_Y_IDXD: bool;

    // The instruction is just one byte. Addressing is implicit
    const IMPLIED: bool;

    // my cause page crossing or not
    const RELATIVE: bool;
}

use std::marker::PhantomData;

pub(crate) use impl_addr_modes;
pub(crate) use impl_deref_mut;

pub(crate) struct Acd<A: AcceptableAddrModes6502> {
    a: PhantomData<A>,
}
pub(crate) struct And<A: AcceptableAddrModes6502> {
    a: PhantomData<A>,
}
pub(crate) struct Cmp<A: AcceptableAddrModes6502> {
    a: PhantomData<A>,
}
pub(crate) struct Eor<A: AcceptableAddrModes6502> {
    a: PhantomData<A>,
}
pub(crate) struct Lda<A: AcceptableAddrModes6502> {
    a: PhantomData<A>,
}
pub(crate) struct Ora<A: AcceptableAddrModes6502> {
    a: PhantomData<A>,
}
pub(crate) struct Sbc<A: AcceptableAddrModes6502> {
    a: PhantomData<A>,
}
pub(crate) struct Sta<A: AcceptableAddrModes6502> {
    a: PhantomData<A>,
}

// impl_addr_modes!(Acd<A>, And<A>, Cmp<A>, Eor<A>, Lda<A>, Ora<A>, Sbc<A>, Sta<A>; 1);

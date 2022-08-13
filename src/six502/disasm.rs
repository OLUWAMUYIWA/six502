pub struct DisAsm<'a> {
    prog: &'a [u8],
}

impl<'a> DisAsm<'a> {
    pub fn new(prog: &'a [u8]) -> Self {
        Self { prog }
    }
}

pub static INSTRUCTIONS: [&str; 256] = [
    "BRK", "ORA izx", "*KIL", "*SLO izx", "*NOP zp", "ORA zp", "ASL zp", "*SLO zp", "PHP",
    "ORA imm", "ASL", "*ANC imm", "*NOP abs", "ORA abs", "ASL abs", "*SLO abs", "BPL rel",
    "ORA izy", "*KIL", "*SLO izy", "*NOP zpx", "ORA zpx", "ASL zpx", "*SLO zpx", "CLC", "ORA aby",
    "*NOP", "*SLO aby", "*NOP abx", "ORA abx", "ASL abx", "*SLO abx", "JSR abs", "AND izx", "*KIL",
    "*RLA izx", "BIT zp", "AND zp", "ROL zp", "*RLA zp", "PLP", "AND imm", "ROL", "*ANC imm",
    "BIT abs", "AND abs", "ROL abs", "*RLA abs", "BMI rel", "AND izy", "*KIL", "*RLA izy",
    "*NOP zpx", "AND zpx", "ROL zpx", "*RLA zpx", "SEC", "AND aby", "*NOP", "*RLA aby", "*NOP abx",
    "AND abx", "ROL abx", "*RLA abx", "RTI", "EOR izx", "*KIL", "*SRE izx", "*NOP zp", "EOR zp",
    "LSR zp", "*SRE zp", "PHA", "EOR imm", "LSR", "*ALR imm", "JMP abs", "EOR abs", "LSR abs",
    "*SRE abs", "BVC rel", "EOR izy", "*KIL", "*SRE izy", "*NOP zpx", "EOR zpx", "LSR zpx",
    "*SRE zpx", "CLI", "EOR aby", "*NOP", "*SRE aby", "*NOP abx", "EOR abx", "LSR abx", "*SRE abx",
    "RTS", "ADC izx", "*KIL", "*RRA izx", "*NOP zp", "ADC zp", "ROR zp", "*RRA zp", "PLA",
    "ADC imm", "ROR", "*ARR imm", "JMP ind", "ADC abs", "ROR abs", "*RRA abs", "BVS rel",
    "ADC izy", "*KIL", "*RRA izy", "*NOP zpx", "ADC zpx", "ROR zpx", "*RRA zpx", "SEI", "ADC aby",
    "*NOP", "*RRA aby", "*NOP abx", "ADC abx", "ROR abx", "*RRA abx", "*NOP imm", "STA izx",
    "*NOP imm", "*SAX izx", "STY zp", "STA zp", "STX zp", "*SAX zp", "DEY", "*NOP imm", "TXA",
    "*XAA imm", "STY abs", "STA abs", "STX abs", "*SAX abs", "BCC rel", "STA izy", "*KIL",
    "*AHX izy", "STY zpx", "STA zpx", "STX zpy", "*SAX zpy", "TYA", "STA aby", "TXS", "*TAS aby",
    "*SHY abx", "STA abx", "*SHX aby", "*AHX aby", "LDY imm", "LDA izx", "LDX imm", "*LAX izx",
    "LDY zp", "LDA zp", "LDX zp", "*LAX zp", "TAY", "LDA imm", "TAX", "*LAX imm", "LDY abs",
    "LDA abs", "LDX abs", "*LAX abs", "BCS rel", "LDA izy", "*KIL", "*LAX izy", "LDY zpx",
    "LDA zpx", "LDX zpy", "*LAX zpy", "CLV", "LDA aby", "TSX", "*LAS aby", "LDY abx", "LDA abx",
    "LDX aby", "*LAX aby", "CPY imm", "CMP izx", "*NOP imm", "*DCP izx", "CPY zp", "CMP zp",
    "DEC zp", "*DCP zp", "INY", "CMP imm", "DEX", "*AXS imm", "CPY abs", "CMP abs", "DEC abs",
    "*DCP abs", "BNE rel", "CMP izy", "*KIL", "*DCP izy", "*NOP zpx", "CMP zpx", "DEC zpx",
    "*DCP zpx", "CLD", "CMP aby", "*NOP", "*DCP aby", "*NOP abx", "CMP abx", "DEC abx", "*DCP abx",
    "CPX imm", "SBC izx", "*NOP imm", "*ISC izx", "CPX zp", "SBC zp", "INC zp", "*ISC zp", "INX",
    "SBC imm", "NOP", "*SBC imm", "CPX abs", "SBC abs", "INC abs", "*ISC abs", "BEQ rel",
    "SBC izy", "*KIL", "*ISC izy", "*NOP zpx", "SBC zpx", "INC zpx", "*ISC zpx", "SED", "SBC aby",
    "*NOP", "*ISC aby", "*NOP abx", "SBC abx", "INC abx", "*ISC abx",
];

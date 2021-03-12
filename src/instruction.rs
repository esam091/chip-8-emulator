use std::convert::TryInto;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Instruction {
    // 00E0
    ClearScreen,

    // 00EE,
    ReturnFromSubroutine,

    // 1NNN
    JumpToAddress(u16),

    // 2NNN,
    CallSubroutineAtAddress(u16),

    // 3XNN
    SkipIfEqual {
        register: u8,
        value: u8,
    },

    // 4XNN
    SkipIfNotEqual {
        register: u8,
        value: u8,
    },

    // 5XY0
    SkipIfRegistersEqual {
        register_x: u8,
        register_y: u8,
    },

    // 6XNN
    SetV {
        register: u8,
        value: u8,
    },

    // 7XNN
    AddToRegister {
        register: u8,
        value: u8,
    },

    // 8XY0
    StoreYToX {
        register_x: u8,
        register_y: u8,
    },

    // 8XY1
    OrRegisters {
        register_x: u8,
        register_y: u8,
    },

    // 8XY2
    AndRegisters {
        register_x: u8,
        register_y: u8,
    },

    // 8XY3
    XorRegisters {
        register_x: u8,
        register_y: u8,
    },

    // 8XY4
    AddRegisters {
        register_x: u8,
        register_y: u8,
    },

    // 8XY5
    SubtractXMinusY {
        register_x: u8,
        register_y: u8,
    },

    // 8XY6

    // 8XY7
    SubtractYMinusX {
        register_x: u8,
        register_y: u8,
    },

    // 8XYE
    ShiftRegisterLeft {
        register_x: u8,
        register_y: u8,
    },

    // 9XY0
    SkipIfRegistersNotEqual {
        register_x: u8,
        register_y: u8,
    },

    // ANNN
    StoreAddrToI(u16),

    // BNNN

    // CXNN
    SetRandomNumber {
        register: u8,
        mask: u8,
    },

    // DXYN
    Draw {
        register_x: u8,
        register_y: u8,
        bytes: u8,
    },

    // EX9E
    // EXA1

    // FX07
    // FX0A
    // FX15
    // FX18

    // FX1E
    AddRegisterToI(u8),
    // FX29
    // FX33
    // FX55
    // FX65
}

fn split_opcode(instruction: u16) -> (u8, u8, u8, u8) {
    let code1 = instruction & 0xf;
    let code2 = (instruction & 0x00f0) >> 4;
    let code3 = (instruction & 0x0f00) >> 8;
    let code4 = (instruction & 0xf000) >> 12;

    return (
        code4.try_into().unwrap(),
        code3.try_into().unwrap(),
        code2.try_into().unwrap(),
        code1.try_into().unwrap(),
    );
}

fn combine_nibble2(a: u8, b: u8) -> u8 {
    (a << 4) ^ b
}

fn combine_nibble3(a: u8, b: u8, c: u8) -> u16 {
    ((a as u16) << 8) ^ ((b as u16) << 4) ^ c as u16
}

pub fn parse_opcode(instruction: u16) -> Option<Instruction> {
    match split_opcode(instruction) {
        (0x0, 0x0, 0xe, 0x0) => Some(Instruction::ClearScreen),
        (0x0, 0x0, 0xe, 0xe) => Some(Instruction::ReturnFromSubroutine),
        (0x2, a, b, c) => Some(Instruction::CallSubroutineAtAddress(combine_nibble3(
            a, b, c,
        ))),
        (0xa, a, b, c) => Some(Instruction::StoreAddrToI(combine_nibble3(a, b, c))),
        (0x6, register, a, b) => Some(Instruction::SetV {
            register,
            value: combine_nibble2(a, b),
        }),
        (0xd, register_x, register_y, bytes) => Some(Instruction::Draw {
            register_x,
            register_y,
            bytes,
        }),
        (0x7, register, a, b) => Some(Instruction::AddToRegister {
            register,
            value: combine_nibble2(a, b),
        }),
        (0x1, a, b, c) => Some(Instruction::JumpToAddress(combine_nibble3(a, b, c))),
        (0x3, register, a, b) => Some(Instruction::SkipIfEqual {
            register,
            value: combine_nibble2(a, b),
        }),
        (0x4, register, a, b) => Some(Instruction::SkipIfNotEqual {
            register,
            value: combine_nibble2(a, b),
        }),
        (0x5, register_x, register_y, 0) => Some(Instruction::SkipIfRegistersEqual {
            register_x,
            register_y,
        }),
        (0x8, register_x, register_y, 0) => Some(Instruction::StoreYToX {
            register_x,
            register_y,
        }),
        (0x8, register_x, register_y, 1) => Some(Instruction::OrRegisters {
            register_x,
            register_y,
        }),
        (0x8, register_x, register_y, 2) => Some(Instruction::AndRegisters {
            register_x,
            register_y,
        }),
        (0x8, register_x, register_y, 3) => Some(Instruction::XorRegisters {
            register_x,
            register_y,
        }),
        (0x8, register_x, register_y, 4) => Some(Instruction::AddRegisters { register_x, register_y }),
        (0x8, register_x, register_y, 5) => Some(Instruction::SubtractXMinusY { register_x, register_y }),
        (0x8, register_x, register_y, 7) => Some(Instruction::SubtractYMinusX { register_x, register_y }),
        (0x8, register_x, register_y, 0xe) => Some(Instruction::ShiftRegisterLeft { register_x, register_y }),
        (0x9, register_x, register_y, 0) => Some(Instruction::SkipIfRegistersNotEqual { register_x, register_y }),
        (0xc, register, a, b) => Some(Instruction::SetRandomNumber {
            register,
            mask: combine_nibble2(a, b),
        }),
        (0xf, register, 0x1, 0xe) => Some(Instruction::AddRegisterToI(register)),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn split_test() {
        assert_eq!(split_opcode(0xabcd), (0xa, 0xb, 0xc, 0xd));
        assert_eq!(split_opcode(0x839a), (0x8, 0x3, 0x9, 0xa));
    }

    #[test]
    fn opcode_test() {
        let instructions_and_opcodes: Vec<(u16, Instruction)> = vec![
            (0x00e0, Instruction::ClearScreen),
            (0xa22a, Instruction::StoreAddrToI(0x22a)),
            (
                0x600c,
                Instruction::SetV {
                    register: 0,
                    value: 0x0c,
                },
            ),
            (
                0xd01f,
                Instruction::Draw {
                    register_x: 0,
                    register_y: 1,
                    bytes: 0xf,
                },
            ),
            (
                0x7009,
                Instruction::AddToRegister {
                    register: 0,
                    value: 0x09,
                },
            ),
            (0x1228, Instruction::JumpToAddress(0x228)),
            (
                0x3c00,
                Instruction::SkipIfEqual {
                    register: 0xc,
                    value: 0x00,
                },
            ),
            (
                0x4040,
                Instruction::SkipIfNotEqual {
                    register: 0,
                    value: 0x40,
                },
            ),
            (0xf21e, Instruction::AddRegisterToI(2)),
            (0x221a, Instruction::CallSubroutineAtAddress(0x21a)),
            (0x00ee, Instruction::ReturnFromSubroutine),
            (
                0xcc01,
                Instruction::SetRandomNumber {
                    register: 0xc,
                    mask: 0x01,
                },
            ),
            (
                0x5ab0,
                Instruction::SkipIfRegistersEqual {
                    register_x: 0xa,
                    register_y: 0xb,
                },
            ),
            (
                0x8cd1,
                Instruction::OrRegisters {
                    register_x: 0xc,
                    register_y: 0xd,
                },
            ),
            (
                0x83a2,
                Instruction::AndRegisters {
                    register_x: 0x3,
                    register_y: 0xa,
                },
            ),
            (
                0x8633,
                Instruction::XorRegisters {
                    register_x: 0x6,
                    register_y: 0x3,
                },
            ),
            (
                0x8fa4,
                Instruction::AddRegisters {
                    register_x: 0xf,
                    register_y: 0xa,
                }
            ),
            (
                0x8fa5,
                Instruction::SubtractXMinusY {
                    register_x: 0xf,
                    register_y: 0xa,
                }
            ),
            (
                0x8cd7,
                Instruction::SubtractYMinusX {
                    register_x: 0xc,
                    register_y: 0xd,
                }
            ),
            (
                0x9cf0,
                Instruction::SkipIfRegistersNotEqual {
                    register_x: 0xc,
                    register_y: 0xf,
                }
            ),
            (0x8cae, Instruction::ShiftRegisterLeft { register_x: 0xc, register_y: 0xa }),
        ];

        for (instruction, opcode) in instructions_and_opcodes {
            assert_eq!(
                parse_opcode(instruction),
                Some(opcode),
                "Expecting instruction {:#04x?} to translate to opcode {:#04x?}",
                instruction,
                opcode
            );
        }
    }
}

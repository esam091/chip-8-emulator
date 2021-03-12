use std::convert::TryInto;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Instruction {
    // 00E0
    ClearScreen,

    // ANNN
    StoreAddrToI(u16),

    // 6XNN
    SetV {
        register: u8,
        value: u8,
    },

    // DXYN
    Draw {
        register_x: u8,
        register_y: u8,
        bytes: u8,
    },

    // 7XNN
    AddToRegister {
        register: u8,
        value: u8,
    },

    // 1NNN
    JumpToAddress(u16),

    // 3XNN
    SkipIfEqual {
        register: u8,
        value: u8,
    },

    // 0NNN
    ExecuteSubroutine(u16),

    // 8XY0
    StoreYToX {
        register_x: u8,
        register_y: u8,
    },
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
        (0x0, a, b, c) => Some(Instruction::ExecuteSubroutine(combine_nibble3(a, b, c))),
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
        (0x8, register_x, register_y, 0) => Some(Instruction::StoreYToX {
            register_x,
            register_y,
        }),
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
            (0x0038, Instruction::ExecuteSubroutine(0x038)),
            (
                0x8320,
                Instruction::StoreYToX {
                    register_x: 3,
                    register_y: 2,
                },
            ),
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

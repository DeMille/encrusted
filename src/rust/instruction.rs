use std::fmt;
use std::hash;

enum_from_primitive! {
    #[allow(non_camel_case_types)]
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Opcode {
        // Two-operand opcodes (2OP)
        OP2_1  = 1,  OP2_2  = 2,  OP2_3  = 3,  OP2_4  = 4,  OP2_5  = 5,  OP2_6  = 6,
        OP2_7  = 7,  OP2_8  = 8,  OP2_9  = 9,  OP2_10 = 10, OP2_11 = 11, OP2_12 = 12,
        OP2_13 = 13, OP2_14 = 14, OP2_15 = 15, OP2_16 = 16, OP2_17 = 17, OP2_18 = 18,
        OP2_19 = 19, OP2_20 = 20, OP2_21 = 21, OP2_22 = 22, OP2_23 = 23, OP2_24 = 24,
        OP2_25 = 25, OP2_26 = 26, OP2_27 = 27, OP2_28 = 28,
        // One-operand opcodes (1OP)
        OP1_128 = 128, OP1_129 = 129, OP1_130 = 130, OP1_131 = 131, OP1_132 = 132,
        OP1_133 = 133, OP1_134 = 134, OP1_135 = 135, OP1_136 = 136, OP1_137 = 137,
        OP1_138 = 138, OP1_139 = 139, OP1_140 = 140, OP1_141 = 141, OP1_142 = 142,
        OP1_143 = 143,
        // Zero-operand opcodes (0OP)
        OP0_176 = 176, OP0_177 = 177, OP0_178 = 178, OP0_179 = 179, OP0_180 = 180,
        OP0_181 = 181, OP0_182 = 182, OP0_183 = 183, OP0_184 = 184, OP0_185 = 185,
        OP0_186 = 186, OP0_187 = 187, OP0_188 = 188, OP0_189 = 189, OP0_191 = 191,
        // Variable-operand opcodes (VAR)
        VAR_224 = 224, VAR_225 = 225, VAR_226 = 226, VAR_227 = 227, VAR_228 = 228,
        VAR_229 = 229, VAR_230 = 230, VAR_231 = 231, VAR_232 = 232, VAR_233 = 233,
        VAR_234 = 234, VAR_235 = 235, VAR_236 = 236, VAR_237 = 237, VAR_238 = 238,
        VAR_239 = 239, VAR_240 = 240, VAR_241 = 241, VAR_242 = 242, VAR_243 = 243,
        VAR_244 = 244, VAR_245 = 245, VAR_246 = 246, VAR_247 = 247, VAR_248 = 248,
        VAR_249 = 249, VAR_250 = 250, VAR_251 = 251, VAR_252 = 252, VAR_253 = 253,
        VAR_254 = 254, VAR_255 = 255,
        // Extended opcodes (EXT)
        EXT_1000 = 1000, EXT_1001 = 1001, EXT_1002 = 1002, EXT_1003 = 1003,
        EXT_1004 = 1004, EXT_1005 = 1005, EXT_1006 = 1006, EXT_1007 = 1007,
        EXT_1008 = 1008, EXT_1009 = 1009, EXT_1010 = 1010, EXT_1011 = 1011,
        EXT_1012 = 1012, EXT_1013 = 1013, EXT_1016 = 1016, EXT_1017 = 1017,
        EXT_1018 = 1018, EXT_1019 = 1019, EXT_1020 = 1020, EXT_1021 = 1021,
        EXT_1022 = 1022, EXT_1023 = 1023, EXT_1024 = 1024, EXT_1025 = 1025,
        EXT_1026 = 1026, EXT_1027 = 1027, EXT_1028 = 1028, EXT_1029 = 1029,
    }
}

#[derive(Debug, PartialEq)]
pub enum OperandType {
    Small,
    Large,
    Variable,
    Omitted,
}

impl OperandType {
    pub fn from(bytes: &[u8]) -> Vec<OperandType> {
        bytes
            .iter()
            .fold(Vec::new(), |mut acc, n| {
                acc.push((n & 0b1100_0000) >> 6);
                acc.push((n & 0b0011_0000) >> 4);
                acc.push((n & 0b0000_1100) >> 2);
                acc.push(n & 0b0000_0011);
                acc
            })
            .into_iter()
            .map(|b| match b {
                0b00 => OperandType::Large,
                0b01 => OperandType::Small,
                0b10 => OperandType::Variable,
                0b11 => OperandType::Omitted,
                _ => unreachable!("Can't get operand type of: {:08b}", b),
            })
            .take_while(|t| *t != OperandType::Omitted)
            .collect()
    }
}

#[derive(Debug)]
pub enum Operand {
    Small(u8),
    Large(u16),
    Variable(u8),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Operand::Small(x) => write!(f, "#{:02x}", x),
            Operand::Large(x) => write!(f, "{:04x}", x),
            Operand::Variable(x) => match x {
                0 => write!(f, "sp"),
                1..=15 => write!(f, "local{}", x - 1),
                _ => write!(f, "g{}", x - 16),
            },
        }
    }
}

#[derive(Debug)]
pub struct Branch {
    pub condition: u16,
    pub address: Option<usize>,
    pub returns: Option<u16>,
}

#[derive(Debug)]
pub struct Instruction {
    pub addr: usize,
    pub opcode: Opcode,
    pub name: String,
    pub operands: Vec<Operand>,
    pub store: Option<u8>,
    pub branch: Option<Branch>,
    pub text: Option<String>,
    pub next: usize,
}

impl Instruction {
    pub fn does_store(opcode: Opcode, version: u8) -> bool {
        use self::Opcode::*;

        match opcode {
            // does a store in any version
            OP2_8 | OP2_9 | OP2_15 | OP2_16 | OP2_17 | OP2_18 | OP2_19 | OP2_20 | OP2_21
            | OP2_22 | OP2_23 | OP2_24 | OP2_25 | OP1_129 | OP1_130 | OP1_131 | OP1_132
            | OP1_136 | OP1_142 | VAR_224 | VAR_231 | VAR_236 | VAR_246 | VAR_247 | VAR_248
            | EXT_1000 | EXT_1001 | EXT_1002 | EXT_1003 | EXT_1004 | EXT_1009 | EXT_1010
            | EXT_1019 | EXT_1029 => true,
            // only stores in certain versions
            OP1_143 => version < 5,
            OP0_181 => version == 4, // missing * in spec?
            OP0_182 => version == 4, // missing * in spec?
            OP0_185 => version >= 5,
            VAR_228 => version >= 5,
            VAR_233 => version == 6,
            _ => false,
        }
    }

    pub fn does_branch(opcode: Opcode, version: u8) -> bool {
        use self::Opcode::*;

        match opcode {
            // does a branch in any version
            OP2_1 | OP2_2 | OP2_3 | OP2_4 | OP2_5 | OP2_6 | OP2_7 | OP2_10 | OP1_128 | OP1_129
            | OP1_130 | OP0_189 | OP0_191 | VAR_247 | VAR_255 | EXT_1006 | EXT_1024 | EXT_1027 => {
                true
            }
            // only branches in certain versions
            OP0_181 => version < 4,
            OP0_182 => version < 4,
            _ => false,
        }
    }

    pub fn does_text(opcode: Opcode) -> bool {
        use self::Opcode::*;

        match opcode {
            OP0_178 | OP0_179 => true,
            _ => false,
        }
    }

    pub fn name(opcode: Opcode, version: u8) -> String {
        use self::Opcode::*;

        match opcode {
            OP2_1 => "je",
            OP2_2 => "jl",
            OP2_3 => "jg",
            OP2_4 => "dec_chk",
            OP2_5 => "inc_chk",
            OP2_6 => "jin",
            OP2_7 => "test",
            OP2_8 => "or",
            OP2_9 => "and",
            OP2_10 => "test_attr",
            OP2_11 => "set_attr",
            OP2_12 => "clear_attr",
            OP2_13 => "store",
            OP2_14 => "insert_obj",
            OP2_15 => "loadw",
            OP2_16 => "loadb",
            OP2_17 => "get_prop",
            OP2_18 => "get_prop_addr",
            OP2_19 => "get_next_prop",
            OP2_20 => "add",
            OP2_21 => "sub",
            OP2_22 => "mul",
            OP2_23 => "div",
            OP2_24 => "mod",
            OP2_25 => "call_2s",
            OP2_26 => "call_2n",
            OP2_27 => "set_colour",
            OP2_28 => "throw",
            OP1_128 => "jz",
            OP1_129 => "get_sibling",
            OP1_130 => "get_child",
            OP1_131 => "get_parent",
            OP1_132 => "get_prop_len",
            OP1_133 => "inc",
            OP1_134 => "dec",
            OP1_135 => "print_addr",
            OP1_136 => "call_1s",
            OP1_137 => "remove_obj",
            OP1_138 => "print_obj",
            OP1_139 => "ret",
            OP1_140 => "jump",
            OP1_141 => "print_paddr",
            OP1_142 => "load",
            // actually 2 different operations:
            OP1_143 => if version < 4 {
                "not"
            } else {
                "call_1n"
            },
            OP0_176 => "rtrue",
            OP0_177 => "rfalse",
            OP0_178 => "print",
            OP0_179 => "print_ret",
            OP0_180 => "nop",
            OP0_181 => "save",
            OP0_182 => "restore",
            OP0_183 => "restart",
            OP0_184 => "ret_popped",
            // actually 2 different operations:
            OP0_185 => if version < 4 {
                "pop"
            } else {
                "catch"
            },
            OP0_186 => "quit",
            OP0_187 => "new_line",
            OP0_188 => "show_status",
            OP0_189 => "verify",
            OP0_191 => "piracy",
            // "call" is the same as "call_vs" (name changed to remove ambiguity)
            VAR_224 => if version < 4 {
                "call"
            } else {
                "call_vs"
            },
            VAR_225 => "storew",
            VAR_226 => "storeb",
            VAR_227 => "put_prop",
            // "sread", "aread", plain "read" are really all the same thing:
            VAR_228 => if version < 4 {
                "sread"
            } else {
                "aread"
            },
            VAR_229 => "print_char",
            VAR_230 => "print_num",
            VAR_231 => "random",
            VAR_232 => "push",
            VAR_233 => "pull",
            VAR_234 => "split_window",
            VAR_235 => "set_window",
            VAR_236 => "call_vs2",
            VAR_237 => "erase_window",
            VAR_238 => "erase_line",
            VAR_239 => "set_cursor",
            VAR_240 => "get_cursor",
            VAR_241 => "set_text_style",
            VAR_242 => "buffer_mode",
            VAR_243 => "output_stream",
            VAR_244 => "input_stream",
            VAR_245 => "sound_effect",
            VAR_246 => "read_char",
            VAR_247 => "scan_table",
            VAR_248 => "not",
            VAR_249 => "call_vn",
            VAR_250 => "call_vn2",
            VAR_251 => "tokenise",
            VAR_252 => "encode_text",
            VAR_253 => "copy_table",
            VAR_254 => "print_table",
            VAR_255 => "check_arg_count",
            EXT_1000 => "save",
            EXT_1001 => "restore",
            EXT_1002 => "log_shift",
            EXT_1003 => "art_shift",
            EXT_1004 => "set_font",
            EXT_1005 => "draw_picture",
            EXT_1006 => "picture_data",
            EXT_1007 => "erase_picture",
            EXT_1008 => "set_margins",
            EXT_1009 => "save_undo",
            EXT_1010 => "restore_undo",
            EXT_1011 => "print_unicode",
            EXT_1012 => "check_unicode",
            EXT_1013 => "set_true_colour",
            EXT_1016 => "move_window",
            EXT_1017 => "window_size",
            EXT_1018 => "window_style",
            EXT_1019 => "get_wind_prop",
            EXT_1020 => "scroll_window",
            EXT_1021 => "pop_stack",
            EXT_1022 => "read_mouse",
            EXT_1023 => "mouse_window",
            EXT_1024 => "push_stack",
            EXT_1025 => "put_wind_prop",
            EXT_1026 => "print_form",
            EXT_1027 => "make_menu",
            EXT_1028 => "picture_table",
            EXT_1029 => "buffer_screen",
        }.to_string()
    }
}

impl Instruction {
    pub fn advances(&self) -> bool {
        use self::Opcode::*;

        // Some instructions never advance to the next instruction:
        // throw, ret, jump, rtrue, rfalse, print_ret, restart, and ret_popped
        match self.opcode {
            OP2_28 | OP1_139 | OP1_140 | OP0_176 | OP0_177 | OP0_179 | OP0_183 | OP0_184
            | OP0_186 => false,
            _ => true,
        }
    }

    pub fn does_call(&self, version: u8) -> bool {
        use self::Opcode::*;

        match self.opcode {
            OP2_25 | OP2_26 | OP1_136 | VAR_224 | VAR_236 | VAR_249 | VAR_250 => true,
            OP1_143 => version >= 4,
            _ => false,
        }
    }

    pub fn should_advance(&self, version: u8) -> bool {
        !self.does_call(version) && self.opcode != Opcode::OP0_181 && self.opcode != Opcode::OP0_182
    }
}

impl hash::Hash for Instruction {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        state.write_usize(self.addr);
        state.finish();
    }
}

impl PartialEq for Instruction {
    fn eq(&self, other: &Instruction) -> bool {
        self.addr == other.addr
    }
}

impl Eq for Instruction {}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:5x}: {:16}", self.addr, self.name)?;

        for op in &self.operands {
            write!(f, " {}", op)?;
        }

        if let Some(x) = self.store {
            match x {
                0 => write!(f, " -> sp"),
                1..=15 => write!(f, " -> local{}", x - 1),
                _ => write!(f, " -> g{}", x - 16),
            }?;
        };

        if let Some(Branch {
            address,
            returns,
            condition,
        }) = self.branch
        {
            match (address, returns, condition) {
                (Some(addr), _, 1) => write!(f, " ?{:04x}", addr),
                (Some(addr), _, 0) => write!(f, " ?~{:04x}", addr),
                (None, Some(1), 1) => write!(f, " ?rtrue"),
                (None, Some(1), 0) => write!(f, " ?~rtrue"),
                (None, Some(0), 1) => write!(f, " ?rfalse"),
                (None, Some(0), 0) => write!(f, " ?~rfalse"),
                _ => write!(f, ""),
            }?;
        };

        if let Some(ref text) = self.text {
            write!(f, " {}", text)?;
        };

        write!(f, "")
    }
}

use std::fmt;

#[derive(Debug)]
pub struct Frame {
    stack: Vec<u16>,
    locals: Vec<u16>,
    pub arg_count: u8,
    pub resume: usize,
    pub store: Option<u8>,
}

impl Frame {
    pub fn new(resume: usize, store: Option<u8>, mut locals: Vec<u16>, arguments: &[u16]) -> Frame {
        for i in 0..locals.len() {
            if arguments.len() > i {
                locals[i] = arguments[i];
            }
        }

        Frame {
            stack: Vec::new(),
            arg_count: arguments.len() as u8,
            locals,
            resume,
            store,
        }
    }

    pub fn empty() -> Frame {
        Frame {
            stack: Vec::new(),
            locals: Vec::new(),
            arg_count: 0,
            resume: 0,
            store: None,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Frame {
        // pc addr as 3 bytes
        let mut resume = 0;
        resume += (bytes[0] as usize) << 16;
        resume += (bytes[1] as usize) << 8;
        resume += bytes[2] as usize;

        let flags = bytes[3];
        let has_store = (flags & 0b0001_0000) == 0;
        let num_locals = flags & 0b0000_1111;

        let store = if has_store { Some(bytes[4]) } else { None };

        let mask = bytes[5];
        let mut arg_count = 0;

        for bit in 0..7 {
            if (mask & (1 << bit)) != 0 {
                arg_count += 1;
            }
        }

        let mut stack_length = 0;
        stack_length += u16::from(bytes[6]) << 8;
        stack_length += u16::from(bytes[7]);

        let mut locals = Vec::new();
        let mut stack = Vec::new();
        let mut index = 8; // locals start @ byte 8

        for offset in 0..num_locals as usize {
            let mut word = 0;
            word += u16::from(bytes[index + offset * 2]) << 8;
            word += u16::from(bytes[index + offset * 2 + 1]);

            locals.push(word);
        }

        index += num_locals as usize * 2; // stack values start after locals

        for offset in 0..stack_length as usize {
            let mut word = 0;
            word += u16::from(bytes[index + offset * 2]) << 8;
            word += u16::from(bytes[index + offset * 2 + 1]);

            stack.push(word);
        }

        Frame {
            stack,
            locals,
            arg_count,
            resume,
            store,
        }
    }

    pub fn read_local(&self, index: u8) -> u16 {
        let index = index as usize;

        if index > self.locals.len() {
            panic!("Trying to read out of bounds local @: {}", index);
        }

        self.locals[index]
    }

    pub fn write_local(&mut self, index: u8, value: u16) {
        let index = index as usize;

        if index > self.locals.len() {
            panic!("Trying to write out of bounds local @: {}", index);
        }

        self.locals[index] = value;
    }

    pub fn stack_push(&mut self, value: u16) {
        self.stack.push(value);
    }

    pub fn stack_pop(&mut self) -> u16 {
        self.stack.pop().expect("Can't pop off an empty stack!")
    }

    pub fn stack_peek(&self) -> u16 {
        *self.stack.last().expect("Can't peek on an empty stack!")
    }

    pub fn to_string(&self) -> String {
        let stringify = |values: &Vec<u16>| {
            let mut out = String::from("[");

            for (i, val) in values.iter().enumerate() {
                if i != 0 {
                    out.push_str(", ")
                }
                out.push_str(&format!("{:04x}", val));
            }

            out.push_str("]");
            out
        };

        format!(
            "Locals: {} Stack: {} -> {:?} @ {:04x}",
            &stringify(&self.locals),
            &stringify(&self.stack),
            self.store,
            self.resume
        )
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // write pc addr as 3 bytes
        bytes.push(((self.resume & 0xFF_0000) >> 16) as u8);
        bytes.push(((self.resume & 0x00_FF00) >> 8) as u8);
        bytes.push((self.resume & 0x00_00FF) as u8);

        let mut flags = self.locals.len() as u8; // 0b0000vvvv
        if self.store.is_some() {
            flags += 0b0001_0000;
        }

        let mut args_supplied = 0b0000_0000;
        for bit in 0..self.arg_count {
            args_supplied |= 1 << bit;
        }

        bytes.push(flags);
        bytes.push(self.store.unwrap_or(0));
        bytes.push(args_supplied);

        let stack_length = self.stack.len();
        bytes.push(((stack_length & 0xFF00) >> 8) as u8);
        bytes.push((stack_length & 0x00FF) as u8);

        self.locals.iter().for_each(|local| {
            bytes.push(((local & 0xFF00) >> 8) as u8);
            bytes.push((local & 0x00FF) as u8);
        });

        self.stack.iter().for_each(|var| {
            bytes.push(((var & 0xFF00) >> 8) as u8);
            bytes.push((var & 0x00FF) as u8);
        });

        bytes
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

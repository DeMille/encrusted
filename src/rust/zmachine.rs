
use std::boxed::Box;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt;
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process;
use std::str;

use base64;
use enum_primitive::FromPrimitive;
use rand;
use rand::{Rng, SeedableRng};
use serde_json;

use buffer::Buffer;
use frame::Frame;
use instruction::Branch;
use instruction::Instruction;
use instruction::Opcode;
use instruction::Operand;
use instruction::OperandType;
use options::Options;
use quetzal::QuetzalSave;
use traits::UI;

#[derive(Debug)]
enum ZStringState {
    Alphabet(usize),
    Abbrev(u8),
    Tenbit1,
    Tenbit2(u8),
}

#[derive(Debug, Serialize)]
pub struct Object {
    number: u16,
    name: String,
    children: Vec<Box<Object>>,
}

impl Object {
    fn new(number: u16, zvm: &Zmachine) -> Box<Object> {
        let mut name = if number > 0 {
            zvm.get_object_name(number)
        } else {
            String::from("(Null Object)")
        };

        if name == "" {
            name += "(No Name)";
        }

        Box::new(Object {
            number,
            name,
            children: Vec::new(),
        })
    }

    fn print_tree(&self, indent: &str, mut depth: u8, is_last: bool) -> String {
        let mut next = String::from(indent);
        let mut out = String::new();

        if depth == 0 {
            out += &format!("{} ({})\n", self.name, self.number);
        } else {
            out += &format!(
                "{}{}── {} ({})\n",
                indent,
                if is_last { "└" } else { "├" },
                self.name,
                self.number
            );

            next += if is_last { "    " } else { "|   " };
        }

        depth += 1;

        for (i, child) in self.children.iter().enumerate() {
            let is_last_child = i == self.children.len() - 1;
            out += &(**child).print_tree(&next, depth, is_last_child);
        }

        out
    }

    fn to_string(&self) -> String {
        if !self.children.is_empty() {
            self.print_tree("", 0, false)
        } else {
            format!("{} ({})", self.name, self.number)
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Debug)]
struct ObjectProperty {
    num: u8,
    len: u8,
    addr: usize,
    next: usize,
}

impl ObjectProperty {
    fn zero() -> ObjectProperty {
        ObjectProperty {
            num: 0,
            addr: 0,
            len: 0,
            next: 0,
        }
    }
}

pub struct Zmachine {
    pub ui: Box< dyn UI>,
    pub options: Options,
    pub instr_log: String,
    version: u8,
    memory: Buffer,
    original_dynamic: Vec<u8>,
    save_dir: String,
    save_name: String,
    static_start: usize,
    routine_offset: usize,
    string_offset: usize,
    alphabet: [Vec<String>; 3],
    abbrev_table: usize,
    separators: Vec<char>,
    dictionary: HashMap<String, usize>,
    frames: Vec<Frame>,
    initial_pc: usize,
    pc: usize,
    globals_addr: usize,
    prop_defaults: usize,
    obj_table_addr: usize,
    obj_size: usize,
    attr_width: usize,
    paused_instr: Option<Instruction>,
    current_state: Option<(String, Vec<u8>)>,
    undos: Vec<(String, Vec<u8>)>,
    redos: Vec<(String, Vec<u8>)>,
    rng: rand::XorShiftRng,
}

impl Zmachine {
    pub fn new(data: Vec<u8>, ui: Box<dyn UI>, options: Options) -> Zmachine {
        let memory = Buffer::new(data);

        let version = memory.read_byte(0x00);
        let initial_pc = memory.read_word(0x06) as usize;
        let prop_defaults = memory.read_word(0x0A) as usize;
        let static_start = memory.read_word(0x0E) as usize;

        let alphabet = if version >= 5 {
            Zmachine::load_alphabet(&memory)
        } else {
            Zmachine::default_alphabet()
        };

        let mut zvm = Zmachine {
            version,
            ui,
            save_dir: options.save_dir.clone(),
            save_name: format!("{}.sav", &options.save_name),
            instr_log: String::new(),
            original_dynamic: memory.slice(0, static_start).to_vec(),
            globals_addr: memory.read_word(0x0C) as usize,
            routine_offset: memory.read_word(0x28) as usize,
            string_offset: memory.read_word(0x2A) as usize,
            static_start,
            initial_pc,
            pc: initial_pc,
            frames: vec![Frame::empty()],
            alphabet,
            abbrev_table: memory.read_word(0x18) as usize,
            separators: Vec::new(),
            dictionary: HashMap::new(),
            prop_defaults,
            obj_table_addr: prop_defaults + (if version <= 3 { 31 } else { 63 }) * 2,
            obj_size: if version <= 3 { 9 } else { 14 },
            attr_width: if version <= 3 { 4 } else { 6 },
            paused_instr: None,
            current_state: None,
            undos: Vec::new(),
            redos: Vec::new(),
            rng: rand::SeedableRng::from_seed(options.rand_seed.clone()),
            memory,
            options,
        };

        // read into dictionary & word separators
        zvm.populate_dictionary();

        zvm
    }

    #[allow(dead_code)]
    fn calculate_checksum(memory: &Buffer) -> u16 {
        let mut sum: usize = 0;
        let len = memory.read_byte(0x1A) as usize * 2;

        for i in 0x40..len {
            sum += memory.read_byte(i) as usize;
        }

        (sum % 0x1_0000) as u16
    }

    fn to_alphabet_entry(s: &str) -> Vec<String> {
        s.chars().map(|c| c.to_string()).collect()
    }

    #[allow(non_snake_case)]
    fn default_alphabet() -> [Vec<String>; 3] {
        let A0 = " .....abcdefghijklmnopqrstuvwxyz";
        let A1 = " .....ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let A2 = " ......\n0123456789.,!?_#'\"/\\-:()";

        [
            Zmachine::to_alphabet_entry(A0),
            Zmachine::to_alphabet_entry(A1),
            Zmachine::to_alphabet_entry(A2),
        ]
    }

    #[allow(non_snake_case)]
    fn load_alphabet(memory: &Buffer) -> [Vec<String>; 3] {
        let alphabet_addr = memory.read_word(0x34) as usize;

        if alphabet_addr == 0 {
            Zmachine::default_alphabet()
        } else {
            let A0 = format!(" .....{}", str::from_utf8(memory.read(alphabet_addr, 26)).expect("bad alphabet table A0!"));
            let A1 = format!(" .....{}", str::from_utf8(memory.read(alphabet_addr + 26, 26)).expect("bad alphabet table A1!"));
            // First two characters are ignored and accounted for in our padding.
            let A2 = format!(" ......\n{}", str::from_utf8(memory.read(alphabet_addr + 26 + 26 + 2, 24)).expect("Bad alphabet table A2!"));

            [
                Zmachine::to_alphabet_entry(&A0),
                Zmachine::to_alphabet_entry(&A1),
                Zmachine::to_alphabet_entry(&A2),
            ]
        }
    }

    fn unpack(&self, addr: u16) -> usize {
        let addr = addr as usize;

        match self.version {
            1..=3 => addr * 2,
            4..=7 => addr * 4,
            8 => addr * 8,
            _ => unreachable!(),
        }
    }

    fn unpack_routine_addr(&self, addr: u16) -> usize {
        match self.unpack(addr) {
            x @ 6..=7 => x + self.routine_offset * 8,
            x => x,
        }
    }

    fn unpack_print_paddr(&self, addr: u16) -> usize {
        match self.unpack(addr) {
            x @ 6..=7 => x + self.string_offset * 8,
            x => x,
        }
    }

    fn read_global(&self, index: u8) -> u16 {
        if index > 240 {
            panic!("Can't read global{}!", index);
        }

        let addr = self.globals_addr + index as usize * 2;
        self.memory.read_word(addr)
    }

    fn write_global(&mut self, index: u8, value: u16) {
        if index > 240 {
            panic!("Can't write global{}!", index);
        }

        let addr = self.globals_addr + index as usize * 2;
        self.memory.write_word(addr, value);
    }

    fn read_local(&self, index: u8) -> u16 {
        self.frames
            .last()
            .expect("Can't write local, no frames!")
            .read_local(index)
    }

    fn write_local(&mut self, index: u8, value: u16) {
        self.frames
            .last_mut()
            .expect("Can't write local, no frames!")
            .write_local(index, value);
    }

    fn stack_push(&mut self, value: u16) {
        self.frames
            .last_mut()
            .expect("Can't push to stack, no frames!")
            .stack_push(value);
    }

    fn stack_pop(&mut self) -> u16 {
        self.frames
            .last_mut()
            .expect("Can't pop stack, no frames!")
            .stack_pop()
    }

    fn stack_peek(&mut self) -> u16 {
        self.frames
            .last_mut()
            .expect("Can't peek stack, no frames!")
            .stack_peek()
    }

    fn read_variable(&mut self, index: u8) -> u16 {
        #[allow(unreachable_patterns)]
        match index {
            0 => self.stack_pop(),
            1..=15 => self.read_local(index - 1),
            16..=255 => self.read_global(index - 16),
            _ => unreachable!(),
        }
    }

    fn read_indirect_variable(&mut self, index: u8) -> u16 {
        #[allow(unreachable_patterns)]
        match index {
            0 => self.stack_peek(),
            1..=15 => self.read_local(index - 1),
            16..=255 => self.read_global(index - 16),
            _ => unreachable!(),
        }
    }

    fn write_variable(&mut self, index: u8, value: u16) {
        #[allow(unreachable_patterns)]
        match index {
            0 => self.stack_push(value),
            1..=15 => self.write_local(index - 1, value),
            16..=255 => self.write_global(index - 16, value),
            _ => unreachable!(),
        }
    }

    fn write_indirect_variable(&mut self, index: u8, value: u16) {
        #[allow(unreachable_patterns)]
        match index {
            0 => {
                self.stack_pop();
                self.stack_push(value);
            }
            1..=15 => self.write_local(index - 1, value),
            16..=255 => self.write_global(index - 16, value),
            _ => unreachable!(),
        }
    }

    fn get_abbrev(&self, index: u8) -> String {
        if index > 96 {
            panic!("Bad abbrev index: {}", index);
        }

        let offset = 2 * index as usize;
        let word_addr = self.memory.read_word(self.abbrev_table + offset);
        let addr = word_addr * 2; // "Word addresses are used only in the abbreviations table" - 1.2.2

        self.read_zstring_from_abbrev(addr as usize)
    }

    fn read_zstring_from_abbrev(&self, addr: usize) -> String {
        self.read_zstring_impl(addr, false)
    }

    fn read_zstring(&self, addr: usize) -> String {
        self.read_zstring_impl(addr, true)
    }

    fn read_zstring_impl(&self, addr: usize, allow_abbrevs: bool) -> String {
        use self::ZStringState::*;

        let mut state = Alphabet(0);
        let mut index = addr;
        let mut zstring = String::new();

        // this closure borrows the zstring while it steps through each zchar.
        // (wrapped here in its own scope to force the borrow to end)
        {
            let mut step = |zchar: u8| {
                state = match (zchar, &state) {
                    // the next zchar will be an abbrev index
                    (zch, &Alphabet(_)) if zch >= 1 && zch <= 3 => {
                        assert!(allow_abbrevs, "Abbrev at {} contained recursive abbrev!", addr);
                        Abbrev(zch)
                    }
                    // shift character for the next zchar
                    (4, &Alphabet(_)) => Alphabet(1),
                    (5, &Alphabet(_)) => Alphabet(2),
                    // special 10bit case, next 2 zchars = one 10bit zscii char
                    (6, &Alphabet(2)) => Tenbit1,
                    (_, &Tenbit1) => Tenbit2(zchar),
                    (_, &Tenbit2(first)) => {
                        let letter = ((first << 5) + zchar) as char;
                        zstring.push_str(&letter.to_string());
                        Alphabet(0)
                    }
                    // get the abbrev at this addr
                    (_, &Abbrev(num)) => {
                        let abbrev = self.get_abbrev((num - 1) * 32 + zchar);
                        zstring.push_str(&abbrev);
                        Alphabet(0)
                    }
                    // normal case, adds letter from correct alphabet and resets to A0
                    (_, &Alphabet(num)) => {
                        let letter = &self.alphabet[num][zchar as usize];
                        zstring.push_str(letter);
                        Alphabet(0)
                    }
                };
            };

            // 3 zchars per each 16 bit word + a "stop" bit on top
            // 0 10101 01010 10101
            loop {
                let word = self.memory.read_word(index);
                index += 2;

                step(((word >> 10) & 0b0001_1111) as u8);
                step(((word >> 5) & 0b0001_1111) as u8);
                step((word & 0b0001_1111) as u8);

                // stop bit
                if word & 0x8000 != 0 {
                    break;
                }
            }
        } // <- drop process closure, ending zstring borrow

        zstring
    }

    // reads the ENCODED byte length of a zstring, how many consecutive
    // bytes in memory it is (not just the number of characters)
    fn zstring_length(&self, addr: usize) -> usize {
        let mut length = 0;

        loop {
            let word = self.memory.read_word(addr + length);
            length += 2;

            // stop bit
            if word & 0x8000 != 0 {
                break;
            }
        }

        length
    }

    fn populate_dictionary(&mut self) {
        let dictionary_start = self.memory.read_word(0x08) as usize;
        let mut read = self.memory.get_reader(dictionary_start);

        let separator_count = read.byte();

        for _ in 0..separator_count {
            self.separators.push(read.byte() as char);
        }

        let entry_length = read.byte() as usize;
        let entry_count = read.word() as usize;
        let entry_start = read.position();

        for n in 0..entry_count {
            let addr = entry_start + n * entry_length;
            let entry = self.read_zstring(addr);

            self.dictionary.insert(entry, addr);
        }
    }

    fn check_dict(&self, word: &str) -> usize {
        let length = if self.version <= 3 { 6 } else { 9 };
        let mut short = word.to_string();
        short.truncate(length);

        match self.dictionary.get(&short) {
            Some(addr) => *addr,
            None => 0,
        }
    }

    fn tokenise(&mut self, text: &str, parse_addr: usize) {
        // v1-4 start storing @ byte 1, v5+ start @2;
        let start = if self.version <= 4 { 1 } else { 2 };
        let mut input = String::from(text);
        let mut found = HashMap::new();

        for sep in &self.separators {
            input = input.replace(&sep.to_string(), &format!(" {} ", sep))
        }

        let tokens: Vec<_> = input
            .split_whitespace()
            .filter(|token| !token.is_empty())
            .map(|token| {
                let offset = found.entry(token).or_insert(0);
                let position = text[*offset..].find(token).unwrap();

                let dict_addr = self.check_dict(token);
                let token_addr = *offset + position + start;

                *offset += position + token.len();

                (dict_addr, token.len(), token_addr)
            })
            .collect();

        let mut write = self.memory.get_writer(parse_addr + 1);
        write.byte(tokens.len() as u8);

        tokens.iter().for_each(|&(dict_addr, len, token_addr)| {
            write.word(dict_addr as u16);
            write.byte(len as u8);
            write.byte(token_addr as u8);
        });
    }

    fn get_object_addr(&self, object: u16) -> usize {
        if object == 0 {
            return self.obj_table_addr;
        }

        self.obj_table_addr + ((object as usize - 1) * self.obj_size)
    }

    fn get_object_prop_table_addr(&self, object: u16) -> usize {
        let addr = self.get_object_addr(object)
            // skip attributes
            + self.attr_width
            // ship parent/child/sibling data
            + if self.version <= 3 { 3 } else { 6 };

        // the property table address is in the next word:
        self.memory.read_word(addr) as usize
    }

    // Object name is found at the start the object's property table:
    //   text-length   text of short name of object
    //   ---byte----   --some even number of bytes--
    fn get_object_name(&self, object: u16) -> String {
        let addr = self.get_object_prop_table_addr(object);
        let text_length = self.memory.read_byte(addr);

        if text_length > 0 {
            self.read_zstring(addr + 1)
        } else {
            String::new()
        }
    }

    fn get_parent(&self, object: u16) -> u16 {
        if object == 0 {
            return 0;
        }

        let addr = self.get_object_addr(object) + self.attr_width;

        if self.version <= 3 {
            u16::from(self.memory.read_byte(addr))
        } else {
            self.memory.read_word(addr)
        }
    }

    fn set_parent(&mut self, object: u16, parent: u16) {
        let addr = self.get_object_addr(object) + self.attr_width;

        if self.version <= 3 {
            self.memory.write_byte(addr, parent as u8);
        } else {
            self.memory.write_word(addr, parent);
        }
    }

    fn get_sibling(&self, object: u16) -> u16 {
        if object == 0 {
            return 0;
        }

        let addr = self.get_object_addr(object) + self.attr_width;

        if self.version <= 3 {
            u16::from(self.memory.read_byte(addr + 1))
        } else {
            self.memory.read_word(addr + 2)
        }
    }

    fn set_sibling(&mut self, object: u16, sibling: u16) {
        let addr = self.get_object_addr(object) + self.attr_width;

        if self.version <= 3 {
            self.memory.write_byte(addr + 1, sibling as u8);
        } else {
            self.memory.write_word(addr + 2, sibling);
        }
    }

    fn get_child(&self, object: u16) -> u16 {
        if object == 0 {
            return 0;
        }

        let addr = self.get_object_addr(object) + self.attr_width;

        if self.version <= 3 {
            u16::from(self.memory.read_byte(addr + 2))
        } else {
            self.memory.read_word(addr + 4)
        }
    }

    fn set_child(&mut self, object: u16, child: u16) {
        let addr = self.get_object_addr(object) + self.attr_width;

        if self.version <= 3 {
            self.memory.write_byte(addr + 2, child as u8);
        } else {
            self.memory.write_word(addr + 4, child);
        }
    }

    fn remove_obj(&mut self, object: u16) {
        let parent = self.get_parent(object);
        if parent == 0 {
            return;
        }

        // fix the tree to patch any holes:
        // 1- if the obj is the first child, make the obj's sibling the new child
        // 2- otherwise, connect the two siblings on each side of the obj
        let parents_first_child = self.get_child(parent);
        let younger_sibling = self.get_sibling(object);

        fn get_older(this: &Zmachine, obj: u16, prev: u16) -> u16 {
            let next = this.get_sibling(prev);
            if next == obj {
                prev
            } else {
                get_older(this, obj, next)
            }
        }

        if object == parents_first_child {
            // fix the parent / first child relationship, upgrade the younger sibling
            //   A              A
            //   |        =>    |
            //   B--C--D        C--D
            self.set_child(parent, younger_sibling);
        } else {
            // fix the hole between two siblings ( A--B--C  ->  A--C )
            let older_sibling = get_older(self, object, parents_first_child);
            self.set_sibling(older_sibling, younger_sibling);
        }

        // remove the object by settings its parent to the null object
        // and clear its sibling reference, since it was moved above
        self.set_parent(object, 0);
        self.set_sibling(object, 0);
    }

    fn insert_obj(&mut self, object: u16, destination: u16) {
        let parents_first_child = self.get_child(destination);

        // skip if object is already in the right place
        if parents_first_child == object {
            return;
        }

        // first remove the object from its position and fix that change
        self.remove_obj(object);

        // set parent/child relationship (object becomes the new first child)
        self.set_parent(object, destination);
        self.set_child(destination, object);

        // move the previous first child into this object's sibling spot
        self.set_sibling(object, parents_first_child);
    }

    fn get_total_object_count(&self) -> u16 {
        // by convention, the property table for object #1 is located AFTER
        // the last object in the object table:
        let obj_table_end = self.get_object_prop_table_addr(1);
        let obj_size = self.attr_width + if self.version <= 3 { 3 } else { 9 } + 2;

        // v1-3 have a max of 255 objects, v4+ can have up to 65535
        ((obj_table_end - self.obj_table_addr) / obj_size) as u16
    }

    fn add_object_children(&self, parent: &mut Object) {
        // follow linked list of siblings to get all children:
        // Parent
        //   |
        // Child -- Sibling -- Sibling -- Sibling ...
        let mut next = self.get_child(parent.number);

        while next > 0 {
            parent.children.push(Object::new(next, self));
            next = self.get_sibling(next);
        }

        // get the children of each child
        for child in &mut parent.children {
            self.add_object_children(&mut *child);
        }
    }

    pub fn get_object_tree(&self) -> Object {
        // start using the INVALID_OBJECT 0 as the root
        let mut root = Object::new(0, self);

        // find all top level objects (objects with no parents)
        for i in 1..self.get_total_object_count() + 1 {
            if self.get_parent(i) == 0 {
                root.children.push(Object::new(i, self));
            }
        }

        // recursively fetch children for each top level object
        for object in &mut root.children {
            self.add_object_children(&mut *object);
        }

        *root
    }

    fn find_object(&self, name: &str) -> Option<u16> {
        for i in 1..self.get_total_object_count() + 1 {
            if self.get_object_name(i).to_lowercase() == name.to_lowercase() {
                return Some(i);
            }
        }

        None
    }

    fn find_yourself(&self) -> Option<u16> {
        self.find_object("cretin")
            .or_else(|| self.find_object("you"))
            .or_else(|| self.find_object("yourself"))
    }

    fn test_attr(&self, object: u16, attr: u16) -> u16 {
        if attr as usize > self.attr_width * 8 {
            panic!("Can't test out-of-bounds attribute: {}", attr);
        }

        let addr = self.get_object_addr(object) + attr as usize / 8;
        let byte = self.memory.read_byte(addr);
        let bit = attr % 8;

        if byte & (128 >> bit) != 0 { 1 } else { 0 }
    }

    fn set_attr(&mut self, object: u16, attr: u16) {
        if attr as usize > self.attr_width * 8 {
            panic!("Can't set out-of-bounds attribute: {}", attr);
        }

        let addr = self.get_object_addr(object) + attr as usize / 8;
        let byte = self.memory.read_byte(addr);
        let bit = attr % 8;

        self.memory.write_byte(addr, byte | (128 >> bit));
    }

    fn clear_attr(&mut self, object: u16, attr: u16) {
        if attr as usize > self.attr_width * 8 {
            panic!("Can't clear out-of-bounds attribute: {}", attr);
        }

        let addr = self.get_object_addr(object) + attr as usize / 8;
        let byte = self.memory.read_byte(addr);
        let bit = attr % 8;

        self.memory.write_byte(addr, byte & !(128 >> bit));
    }

    fn get_default_prop(&self, property_number: u16) -> u16 {
        let word_index = (property_number - 1) as usize;
        let addr = self.prop_defaults + word_index * 2;

        self.memory.read_word(addr)
    }

    fn read_object_prop(&self, addr: usize) -> ObjectProperty {
        let header = self.memory.read_byte(addr);
        let mut len;
        let num;
        let value_addr;

        match self.version {
            1..=3 => {
                num = header % 32;
                len = header / 32 + 1;
                value_addr = addr + 1; // 1 byte header
            }
            _ => {
                num = header & 0b0011_1111; // prop num is bottom 6 bits

                if header & 0b1000_0000 != 0 {
                    len = self.memory.read_byte(addr + 1) & 0b0011_1111;
                    if len == 0 { len = 64; } // Z-Machine standard section 12.4.2.1.1

                    value_addr = addr + 2; // 2 byte header
                } else {
                    len = if header & 0b0100_0000 != 0 { 2 } else { 1 };
                    value_addr = addr + 1; // 1 byte header
                }
            }
        }

        ObjectProperty {
            num,
            len,
            addr: value_addr,
            next: value_addr + len as usize,
        }
    }

    fn find_prop(&self, object: u16, property_number: u16) -> ObjectProperty {
        if property_number == 0 {
            return ObjectProperty::zero();
        }

        let addr = self.get_object_prop_table_addr(object);
        let str_length = self.memory.read_byte(addr) as usize * 2; // words in name
        let first_addr = addr + str_length + 1;

        let property_number = property_number as u8;
        let mut prop = self.read_object_prop(first_addr);

        // linear prop read until property_number is found or until we run out
        // props are listed in decreasing order, check to make
        // sure the requested property even exists
        while prop.num != 0 && prop.num != property_number {
            if property_number > prop.num {
                return ObjectProperty::zero();
            }
            prop = self.read_object_prop(prop.next);
        }

        prop
    }

    fn get_prop_value(&self, object: u16, property_number: u16) -> u16 {
        let prop = self.find_prop(object, property_number);

        if prop.num == 0 {
            self.get_default_prop(property_number)
        } else if prop.len == 1 {
            u16::from(self.memory.read_byte(prop.addr))
        } else {
            self.memory.read_word(prop.addr)
        }
    }

    fn get_prop_addr(&self, object: u16, property_number: u16) -> usize {
        let prop = self.find_prop(object, property_number);

        if prop.num != 0 { prop.addr } else { 0 }
    }

    fn get_prop_len(&self, prop_data_addr: usize) -> u8 {
        // weird required edge case
        if prop_data_addr == 0 {
            return 0;
        }

        // address given is the property DATA, the property HEADER is right before
        let prop_header = self.memory.read_byte(prop_data_addr - 1);

        if self.version <= 3 {
            prop_header / 32 + 1
        } else if prop_header & 0b1000_0000 != 0 {
            // This is already the *second* header byte.
            let len = prop_header & 0b0011_1111;

            if len == 0 { 64 } else { len }
        } else if prop_header & 0b0100_0000 != 0 {
            2
        } else {
            1
        }
    }

    fn get_next_prop(&self, object: u16, property_number: u16) -> u16 {
        // if property 0 is requested, give the first property present
        if property_number == 0 {
            let addr = self.get_object_prop_table_addr(object);
            let str_length = self.memory.read_byte(addr) as usize * 2;
            let first_prop = addr + str_length + 1;

            u16::from(self.read_object_prop(first_prop).num)
        } else {
            let prop = self.find_prop(object, property_number);

            u16::from(self.read_object_prop(prop.next).num)
        }
    }

    fn put_prop(&mut self, object: u16, property_number: u16, value: u16) {
        let prop = self.find_prop(object, property_number);

        if prop.len == 1 {
            self.memory.write_byte(prop.addr, value as u8);
        } else {
            self.memory.write_word(prop.addr, value);
        }
    }

    // Web UI only
    #[allow(dead_code)]
    pub fn get_current_room(&self) -> (u16, String) {
        let num = self.read_global(0);
        let name = self.get_object_name(num);

        (num, name)
    }

    fn get_status(&self) -> (String, String) {
        let num = self.read_global(0);
        let left = self.get_object_name(num);

        // bit 1 in header flags:
        // 0 => score/turns
        // 1 => AM/PM
        let right = if self.memory.read_byte(0x01) & 0b0000_0010 == 0 {
            let score = self.read_global(1) as i16;
            let turns = self.read_global(2);

            format!("{}/{}", score, turns)
        } else {
            let mut hours = self.read_global(1);
            let minutes = self.read_global(2);
            let am_pm = if hours >= 12 { "PM" } else { "AM" };
            if hours > 12 {
                hours -= 12;
            }

            format!("{:02}:{:02} {}", hours, minutes, am_pm)
        };

        (left, right)
    }

    pub fn update_status_bar(&self) {
        // status bar only used in v1-3
        if self.version > 3 {
            return;
        }

        let (left, right) = self.get_status();
        self.ui.set_status_bar(&left, &right);
    }

    fn make_save_state(&self, pc: usize) -> Vec<u8> {
        // save the whole dynamic memory region (between 0 and the start of static)
        let dynamic = self.memory.slice(0, self.static_start);
        let original = self.original_dynamic.as_slice();
        let frames = &self.frames;
        let chksum = self.memory.read_word(0x1c);
        let release = self.memory.read_word(0x02);
        let serial = self.memory.read(0x12, 6);

        QuetzalSave::make(pc, dynamic, original, frames, chksum, release, serial)
    }

    fn restore_state(&mut self, data: &[u8]) {
        let save = QuetzalSave::from_bytes(&data[..], &self.original_dynamic[..]);

        // verify that the save if so the right game and that the memory is ok
        if save.chksum != self.memory.read_word(0x1C) {
            panic!("Invalid save, checksum is different");
        }

        if self.static_start < save.memory.len() {
            panic!("Invalid save, memory is too long");
        }

        self.pc = save.pc;
        self.frames = save.frames;
        self.memory.write(0, save.memory.as_slice());
    }

    pub fn undo(&mut self) -> bool {
        if let Some(ref instr) = self.paused_instr {
            if instr.opcode != Opcode::VAR_228 {
                return false;
            }
        }

        if self.undos.is_empty() {
            self.ui.print("\n[Can't undo that far.]\n");
            return false;
        }

        let new_current = self.undos.pop().unwrap();
        self.redos.push(self.current_state.take().unwrap());

        self.restore_state(new_current.1.as_slice());
        self.current_state = Some(new_current);

        true
    }

    pub fn redo(&mut self) -> bool {
        if let Some(ref instr) = self.paused_instr {
            if instr.opcode != Opcode::VAR_228 {
                return false;
            }
        }

        if self.redos.is_empty() {
            self.ui.print("\n[Nothing to redo.]\n");
            return false;
        }

        let new_current = self.redos.pop().unwrap();
        self.undos.push(self.current_state.take().unwrap());

        self.restore_state(new_current.1.as_slice());
        self.current_state = Some(new_current);

        true
    }

    fn get_arguments(&mut self, operands: &[Operand]) -> Vec<u16> {
        operands
            .iter()
            .map(|operand| match *operand {
                Operand::Small(val) => u16::from(val),
                Operand::Large(val) => val,
                Operand::Variable(val) => self.read_variable(val),
            })
            .collect()
    }

    fn return_from_routine(&mut self, value: u16) {
        let frame = self.frames.pop().expect("Can't pop off last frame!");
        self.pc = frame.resume;

        if let Some(index) = frame.store {
            self.write_variable(index, value);
        }
    }

    fn process_branch(&mut self, branch: &Branch, next: usize, result: u16) {
        let Branch {
            address,
            returns,
            condition,
        } = *branch;
        let result = if result >= 1 { 1 } else { 0 };

        if let Some(index) = address {
            self.pc = if result == condition { index } else { next };
        }

        if let Some(value) = returns {
            if result == condition {
                self.return_from_routine(value);
            } else {
                self.pc = next
            }
        }
    }

    fn process_result(&mut self, instr: &Instruction, value: u16) {
        // store the result if needed
        if let Some(index) = instr.store {
            self.write_variable(index, value);
        }

        // check if we need to branch
        if let Some(ref branch) = instr.branch {
            self.process_branch(branch, instr.next, value);
        } else {
            self.pc = instr.next;
        }
    }

    fn decode_instruction(&self, addr: usize) -> Instruction {
        let mut read = self.memory.get_reader(addr);
        let first = read.byte();

        let btm_4 = |num| num & 0b0000_1111;
        let btm_5 = |num| num & 0b0001_1111;
        let get_types = |bytes: &[u8]| OperandType::from(bytes);

        let get_opcode = |code: u8, offset: u16| {
            let num = u16::from(code) + offset;

            match Opcode::from_u16(num) {
                Some(val) => val,
                None => panic!("Opcode not found: {:?}", num),
            }
        };

        use self::OperandType::*;

        #[allow(unreachable_patterns)]
        let (opcode, optypes) = match first {
            0xbe => (get_opcode(read.byte(), 1000), get_types(&[read.byte()])),
            0x00..=0x1f => (get_opcode(btm_5(first), 0), vec![Small, Small]),
            0x20..=0x3f => (get_opcode(btm_5(first), 0), vec![Small, Variable]),
            0x40..=0x5f => (get_opcode(btm_5(first), 0), vec![Variable, Small]),
            0x60..=0x7f => (get_opcode(btm_5(first), 0), vec![Variable, Variable]),
            0x80..=0x8f => (get_opcode(btm_4(first), 128), vec![Large]),
            0x90..=0x9f => (get_opcode(btm_4(first), 128), vec![Small]),
            0xa0..=0xaf => (get_opcode(btm_4(first), 128), vec![Variable]),
            0xb0..=0xbd | 0xbf => (get_opcode(btm_4(first), 176), vec![]), // OP_0
            0xc0..=0xdf => (get_opcode(btm_5(first), 0), get_types(&[read.byte()])),
            0xe0..=0xff => {
                let opcode = get_opcode(btm_5(first), 224);

                if opcode == Opcode::VAR_236 || opcode == Opcode::VAR_250 {
                    (opcode, get_types(&[read.byte(), read.byte()]))
                } else {
                    (opcode, get_types(&[read.byte()]))
                }
            }
            _ => unreachable!(),
        };

        let operands = optypes
            .iter()
            .map(|optype| match *optype {
                OperandType::Small => Operand::Small(read.byte()),
                OperandType::Large => Operand::Large(read.word()),
                OperandType::Variable => Operand::Variable(read.byte()),
                OperandType::Omitted => unreachable!(),
            })
            .collect();

        let store = if Instruction::does_store(opcode, self.version) {
            Some(read.byte())
        } else {
            None
        };

        let branch = if Instruction::does_branch(opcode, self.version) {
            let byte = read.byte() as usize;
            let condition = if byte & 0b1000_0000 != 0 { 1 } else { 0 };

            let offset = if byte & 0b0100_0000 != 0 {
                byte & 0b0011_1111
            } else {
                ((byte & 0b0011_1111) << 8) + read.byte() as usize
            };

            // the offset (if two bytes) is a 14 bit unsigned int: 2^14 = 16384
            let address = if offset > (16384 / 2) {
                Some(read.position() + offset - 16384 - 2)
            } else {
                Some(read.position() + offset - 2)
            };

            match offset {
                0 => Some(Branch {
                    condition,
                    address: None,
                    returns: Some(0),
                }),
                1 => Some(Branch {
                    condition,
                    address: None,
                    returns: Some(1),
                }),
                _ => Some(Branch {
                    condition,
                    address,
                    returns: None,
                }),
            }
        } else {
            None
        };

        let text = if Instruction::does_text(opcode) {
            Some(self.read_zstring(read.position()))
        } else {
            None
        };

        let text_length = if text.is_some() {
            self.zstring_length(read.position())
        } else {
            0
        };

        let name = Instruction::name(opcode, self.version);
        let next = read.position() + text_length;

        Instruction {
            addr,
            opcode,
            name,
            operands,
            store,
            branch,
            text,
            next,
        }
    }

    pub fn handle_instruction(&mut self, instr: &Instruction) {
        use self::Opcode::*;

        // ~mutably~ gets the arguments (might pop stack)
        let args = self.get_arguments(instr.operands.as_slice());

        if env::var("DEBUG").is_ok() {
            println!("\x1B[97m{}\x1B[0m", instr);
        }

        // Match instructions that return values for storing or branching (or both)
        // `result` is an option. either a matched instruction or none (no match)
        let result = match (instr.opcode, &args[..]) {
            (OP2_1, _) if !args.is_empty() => Some(self.do_je(args[0], &args[1..])),
            (OP2_2, &[a, b]) => Some(self.do_jl(a, b)),
            (OP2_3, &[a, b]) => Some(self.do_jg(a, b)),
            (OP2_4, &[var, value]) => Some(self.do_dec_chk(var, value)),
            (OP2_5, &[var, value]) => Some(self.do_inc_chk(var, value)),
            (OP2_6, &[obj1, obj2]) => Some(self.do_jin(obj1, obj2)),
            (OP2_7, &[map, flags]) => Some(self.do_test(map, flags)),
            (OP2_8, &[a, b]) => Some(self.do_or(a, b)),
            (OP2_9, &[a, b]) => Some(self.do_and(a, b)),
            (OP2_10, &[obj, attr]) => Some(self.do_test_attr(obj, attr)),
            (OP2_15, &[array, index]) => Some(self.do_loadw(array, index)),
            (OP2_16, &[array, index]) => Some(self.do_loadb(array, index)),
            (OP2_17, &[obj, prop]) => Some(self.do_get_prop(obj, prop)),
            (OP2_18, &[obj, prop]) => Some(self.do_get_prop_addr(obj, prop)),
            (OP2_19, &[obj, prop]) => Some(self.do_get_next_prop(obj, prop)),
            (OP2_20, &[a, b]) => Some(self.do_add(a, b)),
            (OP2_21, &[a, b]) => Some(self.do_sub(a, b)),
            (OP2_22, &[a, b]) => Some(self.do_mul(a, b)),
            (OP2_23, &[a, b]) => Some(self.do_div(a, b)),
            (OP2_24, &[a, b]) => Some(self.do_mod(a, b)),
            (OP1_128, &[a]) => Some(self.do_jz(a)),
            (OP1_129, &[obj]) => Some(self.do_get_sibling(obj)),
            (OP1_130, &[obj]) => Some(self.do_get_child(obj)),
            (OP1_131, &[obj]) => Some(self.do_get_parent(obj)),
            (OP1_132, &[addr]) => Some(self.do_get_prop_len(addr)),
            (OP1_142, &[var]) => Some(self.do_load(var)),
            (OP1_143, &[value]) if self.version <= 4 => Some(self.do_not(value)),
            (OP0_189, &[]) => Some(self.do_verify()),
            (OP0_191, &[]) => Some(1), // piracy
            (VAR_231, &[range]) => Some(self.do_random(range)),
            (VAR_233, &[var]) if self.version == 6 => Some(self.do_pull(var)),
            (VAR_248, &[val]) if self.version >= 5 => Some(self.do_not(val)),
            (VAR_255, &[num]) => Some(self.do_check_arg_count(num)),
            (EXT_1002, &[num, places]) => Some(self.do_log_shift(num, places)),
            (EXT_1003, &[num, places]) => Some(self.do_art_shift(num, places)),
            _ => None,
        };

        // If one of the above instructions matched, handle its result by
        // either storing it / branching on it / advancing the program counter.
        // Then return early since this instruction is done.
        if let Some(value) = result {
            self.process_result(instr, value);
            return;
        }

        // All other instructions (don't produce a value, only a side effect)
        match (instr.opcode, &args[..]) {
            (OP2_11, &[obj, attr]) => self.do_set_attr(obj, attr),
            (OP2_12, &[obj, attr]) => self.do_clear_attr(obj, attr),
            (OP2_13, &[var, value]) => self.do_store(var, value),
            (OP2_14, &[obj, dest]) => self.do_insert_obj(obj, dest),
            (OP2_25, &[addr, arg]) => self.do_call(instr, addr, &[arg]), // call_2s
            (OP2_26, &[addr, arg]) => self.do_call(instr, addr, &[arg]), // call_2n
            (OP1_133, &[var]) => self.do_inc(var),
            (OP1_134, &[var]) => self.do_dec(var),
            (OP1_135, &[addr]) => self.do_print_addr(addr),
            (OP1_136, &[addr]) => self.do_call(instr, addr, &[]), // call_1s
            (OP1_137, &[obj]) => self.do_remove_obj(obj),
            (OP1_138, &[obj]) => self.do_print_obj(obj),
            (OP1_139, &[value]) => self.do_ret(value),
            (OP1_140, &[offset]) => self.do_jump(offset, instr),
            (OP1_141, &[addr]) => self.do_print_paddr(addr),
            (OP1_143, &[addr]) if self.version >= 5 => self.do_call(instr, addr, &[]), // call_1n
            (OP0_176, _) => self.do_rtrue(),
            (OP0_177, _) => self.do_rfalse(),
            (OP0_178, _) => self.do_print(instr),
            (OP0_179, _) => self.do_print_ret(instr),
            (OP0_181, _) => self.do_save(instr),
            (OP0_182, _) => self.do_restore(instr),
            (OP0_183, _) => self.do_restart(),
            (OP0_184, _) => self.do_ret_popped(),
            (OP0_185, _) => self.do_pop(),
            (OP0_187, _) => self.do_newline(),
            (OP0_188, _) => self.do_show_status(),
            (VAR_224, _) if !args.is_empty() => self.do_call(instr, args[0], &args[1..]), // call
            (VAR_225, &[array, index, value]) => self.do_storew(array, index, value),
            (VAR_226, &[array, index, value]) => self.do_storeb(array, index, value),
            (VAR_227, &[obj, prop, value]) => self.do_put_prop(obj, prop, value),
            (VAR_228, &[text, parse]) => self.do_sread(instr, text, parse),
            (VAR_229, &[chr]) => self.do_print_char(chr),
            (VAR_230, &[num]) => self.do_print_num(num),
            (VAR_232, &[value]) => self.do_push(value),
            (VAR_233, &[var]) => { self.do_pull(var); }
            (VAR_236, _) if !args.is_empty() => self.do_call(instr, args[0], &args[1..]), // call_vs2
            (VAR_249, _) if !args.is_empty() => self.do_call(instr, args[0], &args[1..]), // call_vn
            (VAR_250, _) if !args.is_empty() => self.do_call(instr, args[0], &args[1..]), // call_vn2

            // special cases to no-op: (input/output streams & sound effects)
            // these might be present in some v3 games but aren't implemented yet
            (VAR_243, _) | (VAR_244, _) | (VAR_245, _) => (),

            _ => panic!(
                "\n\nOpcode not yet implemented: {} ({:?}) @ {:#04x}\n\n",
                instr.name, instr.opcode, self.pc
            ),
        }

        // advance pc to the next instruction
        // (but not for jumps, calls, save/restore, or anything with special needs)
        if instr.advances() && instr.should_advance(self.version) {
            self.pc = instr.next;
        }
    }

    fn is_debug_command(&self, input: &str) -> bool {
        if !input.starts_with('$') {
            return false;
        }

        let parts: Vec<_> = input.split_whitespace().collect();
        let command = parts.first().unwrap();

        let valid = [
            "$dump",
            "$dict",
            "$tree",
            "$room",
            "$you",
            "$find",
            "$object",
            "$parent",
            "$simple",
            "$attrs",
            "$props",
            "$header",
            "$history",
            "$have_attr",
            "$have_prop",
            "$undo",
            "$redo",
            "$redo",
            "$teleport",
            "$steal",
            "$help",
        ];

        valid.contains(command)
    }

    fn print_command_help(&mut self) {
        self.ui.debug(
            "\
            Available debug commands: \n\n\
            $dump               (list stack frames and PC) \n\
            $dict               (show games's dictionary) \n\
            $tree               (list current object tree) \n\
            $room               (show current room's sub-tree) \n\
            $you                (show your sub-tree) \n\
            $find name          (find object number from name) \n\
            $object num/name    (show object's sub-tree) \n\
            $parent num/name    (show object's parent sub-tree) \n\
            $simple num         (object info, simple view) \n\
            $attrs num/name     (list object attributes) \n\
            $props num/name     (list object properties) \n\
            $header             (show header info) \n\
            $history            (list saved states) \n\
            $have_attr num      (list objects that have given attribute enabled) \n\
            $have_prop num      (list objects that have given property) \n\
            $teleport num/name  (teleport to a room) \n\
            $steal num/name     (takes any item) \n\
            $undo \n\
            $redo \n\
            $quit
        ",
        );
    }

    fn handle_debug_command(&mut self, input: &str) -> bool {
        let mut should_ask_again = true;
        let parts: Vec<_> = input.split_whitespace().collect();
        let (command, rest) = parts.split_first().unwrap();
        let arg = &rest.join(" ");

        match *command {
            "$help" => self.print_command_help(),
            "$dump" => self.debug_dump(),
            "$dict" => self.debug_dictionary(),
            "$tree" => self.debug_object_tree(),
            "$room" => self.debug_room(),
            "$you" => self.debug_yourself(),
            "$find" => self.debug_find_object(arg),
            "$object" => self.debug_object(arg),
            "$parent" => self.debug_parent(arg),
            "$attrs" => self.debug_object_attributes(arg),
            "$props" => self.debug_object_properties(arg),
            "$simple" => self.debug_object_simple(arg.parse().unwrap_or(1)),
            "$header" => self.debug_header(),
            "$history" => self.debug_history(),
            "$have_attr" => self.debug_have_attribute(arg),
            "$have_prop" => self.debug_have_property(arg),
            "$steal" => self.debug_steal(arg),
            "$teleport" => self.debug_teleport(arg),
            "$quit" => process::exit(0),
            // if undo/redo fails, should ask for input again
            // if they succeed, do nothing because zmachine state changed
            "$undo" => should_ask_again = !self.undo(),
            "$redo" => should_ask_again = !self.redo(),
            // unrecognized commands should ask for user input again
            _ => {
                should_ask_again = false;
            }
        }

        should_ask_again
    }

    // Terminal UI only
    #[allow(dead_code)]
    pub fn run(&mut self) {
        self.ui.clear();

        // continue instructions until the quit instruction
        loop {
            let instr = self.decode_instruction(self.pc);

            if instr.opcode == Opcode::OP0_186 {
                break;
            }

            self.handle_instruction(&instr);
        }

        self.ui.reset();
    }

    // Web UI only
    #[allow(dead_code)]
    pub fn step(&mut self) -> bool {
        // loop through instructions until user input is needed
        // (saves/restores need a save name, read instructions need user input)
        // Pauses on these instructions and control is passed back to js
        loop {
            let instr = self.decode_instruction(self.pc);

            if self.options.log_instructions {
                write!(self.instr_log, "\n{}", &instr).unwrap();
            }

            match instr.opcode {
                // SAVE
                Opcode::OP0_181 => {
                    let pc = instr.next - 1;
                    let state = self.make_save_state(pc);
                    self.send_save_message("save", &state);

                    // Advance the pc, assuming that the save was successful
                    self.process_save_result(&instr);
                }
                // RESTORE (breaks loop)
                Opcode::OP0_182 => {
                    self.ui.message("restore", "");
                    self.paused_instr = Some(instr);

                    return false;
                }
                // QUIT (breaks loop)
                Opcode::OP0_186 => {
                    // undo 2x - get to the savestate right before the
                    // "are you sure?" dialog box that usually shows up:
                    self.undos.pop();

                    if let Some((_, state)) = self.undos.pop() {
                        self.send_save_message("savestate", &state);
                    }

                    return true; // done == true
                }
                // READ (breaks loop)
                Opcode::VAR_228 => {
                    let state = self.make_save_state(self.pc);
                    self.send_save_message("savestate", &state);

                    // web ui saves current state here BEFORE processing user input
                    let (location, _) = self.get_status();
                    self.current_state = Some((location, state));
                    self.paused_instr = Some(instr);

                    return false;
                }
                _ => {
                    self.handle_instruction(&instr);
                }
            }
        }
    }

    // Web UI only - gives user input to the paused read instruction
    // (passes control back JS afterwards)
    #[allow(dead_code)]
    pub fn handle_input(&mut self, input: String) {
        let instr = self.paused_instr.take().expect(
            "Can't handle input, no paused instruction to resume",
        );

        // handle special debugging commands
        // these inputs shouldn't be processed normally
        if self.is_debug_command(&input) {
            if self.handle_debug_command(&input) {
                self.ui.print("\n>");
            }

            // return execution to JS, which will read user input again:
            return;
        }

        // new input changes timelines, so remove any obsolete redos
        self.redos.clear();

        // move current state into the undo list now
        if self.current_state.is_some() {
            self.undos.push(self.current_state.take().unwrap());
        }

        // explicitly handle read (need to get args first)
        let args = self.get_arguments(instr.operands.as_slice());
        self.do_sread_second(args[0], args[1], input);
        self.pc = instr.next;
    }

    // Web UI only
    #[allow(dead_code)]
    pub fn restore(&mut self, data: &str) {
        let state = base64::decode(&data);

        // cancel restore (sending an empty string or if base64 decode fails)
        if data.is_empty() || state.is_err() {
            let instr = self.paused_instr.take().unwrap();
            self.process_result(&instr, 0);
        } else {
            self.restore_state(state.unwrap().as_slice());
            self.process_restore_result();
        }
    }

    // Web UI only
    // Loads a saved state _without_ processing a restore result (like the above)
    #[allow(dead_code)]
    pub fn load_savestate(&mut self, data: &str) {
        let state = base64::decode(data).unwrap();
        self.restore_state(state.as_slice());
    }

    // Web UI only
    #[allow(dead_code)]
    fn send_save_message(&mut self, msg_type: &str, state: &[u8]) {
        let b64 = base64::encode(&state);

        let (location, info) = self.get_status();
        let status = [&location, " - ", &info].concat();

        let msg_body = serde_json::to_string(&(status, b64)).unwrap();
        self.ui.message(msg_type, &msg_body);
    }
}

// Instruction handlers
impl Zmachine {
    // OP2_1
    fn do_je(&self, a: u16, values: &[u16]) -> u16 {
        if values.iter().any(|x| a == *x) { 1 } else { 0 }
    }

    // OP2_2
    fn do_jl(&self, a: u16, b: u16) -> u16 {
        if (a as i16) < (b as i16) { 1 } else { 0 }
    }

    // OP2_3
    fn do_jg(&self, a: u16, b: u16) -> u16 {
        if (a as i16) > (b as i16) { 1 } else { 0 }
    }

    // OP2_4
    fn do_dec_chk(&mut self, var: u16, value: u16) -> u16 {
        let before = self.read_indirect_variable(var as u8) as i16;
        let after = before.wrapping_sub(1);

        self.write_indirect_variable(var as u8, after as u16);

        if after < (value as i16) { 1 } else { 0 }
    }

    // OP2_5
    fn do_inc_chk(&mut self, var: u16, value: u16) -> u16 {
        let before = self.read_indirect_variable(var as u8) as i16;
        let after = before.wrapping_add(1);

        self.write_indirect_variable(var as u8, after as u16);

        if after > (value as i16) { 1 } else { 0 }
    }

    // OP2_6
    fn do_jin(&self, obj1: u16, obj2: u16) -> u16 {
        if self.get_parent(obj1) == obj2 { 1 } else { 0 }
    }

    // OP2_7
    fn do_test(&self, bitmap: u16, flags: u16) -> u16 {
        if bitmap & flags == flags { 1 } else { 0 }
    }

    // OP2_8
    fn do_or(&self, a: u16, b: u16) -> u16 {
        a | b
    }

    // OP2_9
    fn do_and(&self, a: u16, b: u16) -> u16 {
        a & b
    }

    // OP2_10
    fn do_test_attr(&self, obj: u16, attr: u16) -> u16 {
        self.test_attr(obj, attr)
    }

    // OP2_11
    fn do_set_attr(&mut self, obj: u16, attr: u16) {
        self.set_attr(obj, attr)
    }

    // OP2_12
    fn do_clear_attr(&mut self, obj: u16, attr: u16) {
        self.clear_attr(obj, attr)
    }

    // OP2_13
    fn do_store(&mut self, var: u16, value: u16) {
        self.write_indirect_variable(var as u8, value);
    }

    // OP2_14
    fn do_insert_obj(&mut self, object: u16, destination: u16) {
        self.insert_obj(object, destination);
    }

    // OP2_15
    fn do_loadw(&self, array_addr: u16, index: u16) -> u16 {
        let word_index = index.wrapping_mul(2);
        let word_addr = array_addr.wrapping_add(word_index);

        self.memory.read_word(word_addr as usize)
    }

    // OP2_16
    fn do_loadb(&self, array_addr: u16, index: u16) -> u16 {
        let byte_addr = array_addr.wrapping_add(index);

        u16::from(self.memory.read_byte(byte_addr as usize))
    }

    // OP2_17
    fn do_get_prop(&self, object: u16, property_number: u16) -> u16 {
        self.get_prop_value(object, property_number)
    }

    // OP2_18
    fn do_get_prop_addr(&self, object: u16, property_number: u16) -> u16 {
        self.get_prop_addr(object, property_number) as u16
    }

    // OP2_19
    fn do_get_next_prop(&self, object: u16, property_number: u16) -> u16 {
        self.get_next_prop(object, property_number)
    }

    // OP2_20
    fn do_add(&self, a: u16, b: u16) -> u16 {
        (a as i16).wrapping_add(b as i16) as u16
    }

    // OP2_21
    fn do_sub(&self, a: u16, b: u16) -> u16 {
        (a as i16).wrapping_sub(b as i16) as u16
    }

    // OP2_22
    fn do_mul(&self, a: u16, b: u16) -> u16 {
        (a as i16).wrapping_mul(b as i16) as u16
    }

    // OP2_23
    fn do_div(&self, a: u16, b: u16) -> u16 {
        (a as i16).wrapping_div(b as i16) as u16
    }

    // OP2_24
    fn do_mod(&self, a: u16, b: u16) -> u16 {
        (a as i16 % b as i16) as u16
    }

    // OP1_128
    fn do_jz(&self, a: u16) -> u16 {
        if a == 0 { 1 } else { 0 }
    }

    // OP1_129
    fn do_get_sibling(&self, object: u16) -> u16 {
        self.get_sibling(object)
    }

    // OP1_130
    fn do_get_child(&self, object: u16) -> u16 {
        self.get_child(object)
    }

    // OP1_131
    fn do_get_parent(&self, object: u16) -> u16 {
        self.get_parent(object)
    }

    // OP1_132
    fn do_get_prop_len(&self, addr: u16) -> u16 {
        u16::from(self.get_prop_len(addr as usize))
    }

    // OP1_133
    fn do_inc(&mut self, var: u16) {
        let value = self.read_indirect_variable(var as u8);
        let inc = (value as i16).wrapping_add(1);

        self.write_indirect_variable(var as u8, inc as u16);
    }

    // OP1_134
    fn do_dec(&mut self, var: u16) {
        let value = self.read_indirect_variable(var as u8);
        let dec = (value as i16).wrapping_sub(1);

        self.write_indirect_variable(var as u8, dec as u16);
    }

    // OP1_135
    fn do_print_addr(&mut self, addr: u16) {
        let zstring = self.read_zstring(addr as usize);
        self.ui.print(&zstring);
    }

    // OP1_136 : call_1s

    // OP1_137
    fn do_remove_obj(&mut self, obj: u16) {
        self.remove_obj(obj);
    }

    // OP1_138
    fn do_print_obj(&mut self, obj: u16) {
        let name = self.get_object_name(obj);
        self.ui.print_object(&name);
    }

    // OP1_139
    fn do_ret(&mut self, value: u16) {
        self.return_from_routine(value);
    }

    // OP1_140
    fn do_jump(&mut self, offest: u16, instr: &Instruction) {
        self.pc = if (offest as i16) < 0 {
            instr.next - (-(offest as i16)) as usize - 2
        } else {
            instr.next + offest as usize - 2
        };
    }

    // OP1_141
    fn do_print_paddr(&mut self, addr: u16) {
        let paddr = self.unpack_print_paddr(addr);
        let zstring = self.read_zstring(paddr);
        self.ui.print(&zstring);
    }

    // OP1_142
    fn do_load(&mut self, var: u16) -> u16 {
        self.read_indirect_variable(var as u8)
    }

    // OP1_143
    fn do_not(&self, value: u16) -> u16 {
        !value
    }

    // OP0_176
    fn do_rtrue(&mut self) {
        self.return_from_routine(1);
    }

    // OP0_177
    fn do_rfalse(&mut self) {
        self.return_from_routine(0);
    }

    // OP0_178
    fn do_print(&mut self, instr: &Instruction) {
        let text = instr.text.as_ref().expect("Can't print with no text!");
        self.ui.print(text);
    }

    // OP0_179
    fn do_print_ret(&mut self, instr: &Instruction) {
        let text = instr.text.as_ref().expect("Can't print with no text!");
        self.ui.print(text);
        self.ui.print("\n");
        self.return_from_routine(1);
    }

    // OP0_180 : nop, never actually used

    // OP0_181
    fn do_save(&mut self, instr: &Instruction) {
        let prompt = format!("\nFilename [{}]: ", self.save_name);
        self.ui.print(&prompt);

        let input = self.ui.get_user_input();
        let mut path = PathBuf::from(&self.save_dir);
        let mut file;

        match input.to_lowercase().as_ref() {
            "" | "yes" | "y" => path.push(&self.save_name),
            "no" | "n" | "cancel" => {
                self.process_result(instr, 0);
                return;
            }
            _ => path.push(input),
        }

        if let Ok(handle) = File::create(&path) {
            file = handle;
        } else {
            self.ui.print("Can't save to that file, try another?\n");
            self.process_result(instr, 0);
            return;
        }

        // save file name for next use
        self.save_name = path.file_name().unwrap().to_string_lossy().into_owned();

        // The save PC points to either the save instructions branch data or store
        // data. In either case, this is the last byte of the instruction. (so -1)
        let pc = instr.next - 1;
        let data = self.make_save_state(pc);
        file.write_all(&data[..]).expect("Error saving to file");

        self.process_save_result(instr);
    }

    fn process_save_result(&mut self, instr: &Instruction) {
        // (v1-3): follow branch if needed (value "1" means the save succeeded)
        // (v4+):  or store the value "1" at the give store position
        self.process_result(instr, 1);
    }

    // OP0_182
    fn do_restore(&mut self, instr: &Instruction) {
        let prompt = format!("\nFilename [{}]: ", self.save_name);
        self.ui.print(&prompt);

        let input = self.ui.get_user_input();
        let mut path = PathBuf::from(&self.save_dir);
        let mut data = Vec::new();
        let mut file;

        match input.to_lowercase().as_ref() {
            "" | "yes" | "y" => path.push(&self.save_name),
            "no" | "n" | "cancel" => {
                self.process_result(instr, 0);
                return;
            }
            _ => path.push(input),
        }

        if let Ok(handle) = File::open(&path) {
            file = handle;
        } else {
            self.ui.print("Can't open that file, try another?\n");
            self.process_result(instr, 0);
            return;
        }

        // save file name for next use
        self.save_name = path.file_name().unwrap().to_string_lossy().into_owned();

        // restore program counter position, stack frames, and dynamic memory
        file.read_to_end(&mut data).expect(
            "Error reading save file",
        );
        self.restore_state(data.as_slice());
        self.process_restore_result();
    }

    fn process_restore_result(&mut self) {
        // In versions 1-3 the PC points to the BRANCH data of the save instruction.
        // Saves branch if successful, so follow the branch if the topmost bit
        // (condition bit) of the branch data is set. Otherwise go to the next
        // instruction (the next byte address).
        //
        // In versions 4+ the PC points to the number that the save result should
        // be saved in. (Saves store the value 2 when successful)
        //
        // (note: this logic only applies to the save/restore instructions)
        let byte = self.memory.read_byte(self.pc);

        if self.version <= 3 {
            if byte & 0b1000_0000 != 0 {
                self.pc += (byte & 0b0011_1111) as usize - 2; // follow branch
            } else {
                self.pc += 1; // next instruction
            }
        } else {
            self.pc += 1;
            self.write_variable(byte, 2); // store "we just restored" value
        }
    }

    // OP0_183
    fn do_restart(&mut self) {
        self.pc = self.initial_pc;
        self.frames.clear();
        self.frames.push(Frame::empty());
        self.memory.write(0, self.original_dynamic.as_slice());
    }

    // OP0_184
    fn do_ret_popped(&mut self) {
        let value = self.stack_pop();
        self.return_from_routine(value);
    }

    // OP0_185
    fn do_pop(&mut self) {
        self.stack_pop();
    }

    // OP0_187
    fn do_newline(&mut self) {
        self.ui.print("\n");
    }

    // OP0_188
    fn do_show_status(&self) {
        self.update_status_bar();
    }

    // OP0_189
    fn do_verify(&self) -> u16 {
        1
    }

    // All calls:
    // OP2_25, OP2_26, OP1_136, VAR_224, VAR_236, VAR_249, VAR_250
    // and OP1_143 when version > 3
    //
    // The only difference between the different opcodes is number of arguments
    // and whether or not to store or branch based on the result of the call
    //
    fn do_call(&mut self, instr: &Instruction, addr: u16, args: &[u16]) {
        // weird edge case: addr 0 means do nothing, then store/branch on 0
        if addr == 0 {
            self.process_result(instr, 0);
            return;
        }

        // decode routine / prepopulate routine local variables
        let routine_addr = self.unpack_routine_addr(addr);
        let mut read = self.memory.get_reader(routine_addr);

        let mut locals = Vec::new();
        let count = read.byte();

        for _ in 0..count {
            match self.version {
                1..=4 => locals.push(read.word()),
                _ => locals.push(0),
            };
        }

        let first_instr = read.position();
        let frame = Frame::new(instr.next, instr.store, locals, args);

        self.pc = first_instr;
        self.frames.push(frame);
    }

    // VAR_225
    fn do_storew(&mut self, array_addr: u16, index: u16, value: u16) {
        let word_index = index.wrapping_mul(2);
        let word_addr = array_addr.wrapping_add(word_index);

        self.memory.write_word(word_addr as usize, value);
    }

    // VAR_226
    fn do_storeb(&mut self, array: u16, index: u16, value: u16) {
        let word_addr = array.wrapping_add(index);

        self.memory.write_byte(word_addr as usize, value as u8);
    }

    // VAR_227
    fn do_put_prop(&mut self, obj: u16, prop: u16, value: u16) {
        self.put_prop(obj, prop, value);
    }

    // VAR_228
    fn do_sread(&mut self, instr: &Instruction, text_addr: u16, parse_addr: u16) {
        // need to update the status bar before each read
        self.update_status_bar();
        // add extra space so it doesn't look janky (non-spec)
        self.ui.print(" ");

        let input = self.ui.get_user_input();

        // handle special debugging commands
        // these inputs shouldn't be processed normally
        if self.is_debug_command(&input) {
            if self.handle_debug_command(&input) {
                self.ui.print("\n>");
                self.do_sread(instr, text_addr, parse_addr);
            }

            return;
        }

        self.do_sread_second(text_addr, parse_addr, input);

        // save state JUST after having processed user input
        // new input changes timelines, so remove any obsolete redos
        self.redos.clear();

        // push the current state into the undo list
        if self.current_state.is_some() {
            self.undos.push(self.current_state.take().unwrap());
        }

        // and save the current state
        let location = self.get_object_name(self.read_global(0));
        let state = self.make_save_state(instr.next);
        self.current_state = Some((location, state));
    }

    fn do_sread_second(&mut self, text_addr: u16, parse_addr: u16, mut raw: String) {
        let text_addr = text_addr as usize;
        let parse_addr = parse_addr as usize;

        // versions 1-4 have to store an extra 0, so the max length is 1 less
        let mut max_length = self.memory.read_byte(text_addr as usize);
        if self.version <= 4 {
            max_length -= 1;
        }

        raw.truncate(max_length as usize);
        let input = &raw.to_lowercase();

        let bytes = input.as_bytes();
        let len = bytes.len();

        // ver 1-4 start storing @ byte 1, ending with a terminating 0
        // ver 5+ save the input length @1, start storing @2, and DON'T end with 0
        if self.version <= 4 {
            self.memory.write(text_addr + 1, bytes);
            self.memory.write_byte(text_addr + 1 + len, 0);
        } else {
            self.memory.write_byte(text_addr + 1, len as u8);
            self.memory.write(text_addr + 2, bytes);
        }

        // skip tokenization step if parse_addr is 0
        if parse_addr != 0 {
            self.tokenise(input, parse_addr);
        }
    }

    // VAR_229
    fn do_print_char(&mut self, chr: u16) {
        self.ui.print(&(chr as u8 as char).to_string());
    }

    // VAR_230
    fn do_print_num(&mut self, signed: u16) {
        self.ui.print(&(signed as i16).to_string());
    }

    // VAR_231
    fn do_random(&mut self, range: u16) -> u16 {
        let range = range as i16;

        if range <= 0 {
            self.rng.reseed([range as u32, 0, 0, 0]);
            0
        } else if range == 1 {
            1
        } else {
            (self.rng.gen::<f32>() * f32::from(range)).ceil() as u16
        }
    }

    // VAR_232
    fn do_push(&mut self, value: u16) {
        self.stack_push(value)
    }

    // VAR_233
    fn do_pull(&mut self, var: u16) -> u16 {
        let value = self.stack_pop();
        self.write_indirect_variable(var as u8, value);

        value
    }

    // VAR_248 do_not() (same as OP1_143)

    // VAR_255
    fn do_check_arg_count(&self, num: u16) -> u16 {
        let count = u16::from(
            self.frames
                .last()
                .expect("Can't check arg count, no frames!")
                .arg_count,
        );

        if count >= num { 1 } else { 0 }
    }

    // EXT_1002
    fn do_log_shift(&mut self, number: u16, places: u16) -> u16 {
        let number = number as u32;
        let places = places as i16;

        if places > 0 {
            (number << places) as u16
        } else if places < 0 {
            (number >> -places) as u16
        } else {
            number as u16
        }
    }

    // EXT_1003
    fn do_art_shift(&mut self, number: u16, places: u16) -> u16 {
        let mut number = (number as i16) as i32;
        let places = places as i16;

        if places > 0 {
            number <<= places;
        } else if places < 0 {
            number >>= -places;
        }
        (number as i16) as u16
    }
}

// debug functions
#[allow(dead_code)]
impl Zmachine {
    fn debug_header(&mut self) {
        let version = self.memory.read_byte(0x00);
        let release = self.memory.read_word(0x02);
        let initial_pc = self.memory.read_word(0x06);
        let checksum = self.memory.read_word(0x1C);

        let serial = self.memory.read(0x12, 6).to_vec();
        let ascii = String::from_utf8_lossy(&serial[..]);

        self.ui.debug(&format!(
            "\
             Version: {} \n\
             Release: {} / Serial: {} \n\
             Checksum: {:#x} \n\
             Initial PC: {:#x} \n\
             ",
            version, release, ascii, checksum, initial_pc
        ));
    }

    fn debug_dictionary(&mut self) {
        let mut out = String::new();
        let mut words = self.dictionary.keys().collect::<Vec<_>>();
        words.sort();

        let width = words.iter().fold(0, |longest, word| {
            if word.len() > longest {
                word.len()
            } else {
                longest
            }
        });

        for word in &words {
            write!(out, "{:1$}   ", word, width).unwrap()
        }

        out.push_str("\n");
        self.ui.debug(&out);
    }

    fn debug_dump(&mut self) {
        let mut out = String::new();
        writeln!(out, "PC @ {}", self.pc).unwrap();

        for frame in &self.frames {
            writeln!(out, "{}", frame).unwrap();
        }

        self.ui.debug(&out);
    }

    fn debug_object_simple(&mut self, obj_num: u16) {
        self.ui.debug(&format!("\nObject #{}", obj_num));

        let addr = self.get_object_addr(obj_num);
        let mut read = self.memory.get_reader(addr);

        if self.version <= 3 {
            self.ui.debug(&format!(
                "Attributes: {:08b} {:08b} {:08b} {:08b}",
                read.byte(),
                read.byte(),
                read.byte(),
                read.byte()
            ));

            self.ui.debug(&format!(
                "Parent: {}, Sibling: {}, Child: {}",
                read.byte(),
                read.byte(),
                read.byte()
            ));
        } else {
            self.ui.debug(&format!(
                "Attributes: {:08b} {:08b} {:08b} {:08b} {:08b} {:08b}",
                read.byte(),
                read.byte(),
                read.byte(),
                read.byte(),
                read.byte(),
                read.byte()
            ));

            self.ui.debug(&format!(
                "Parent: {}, Sibling: {}, Child: {}",
                read.word(),
                read.word(),
                read.word()
            ));
        }

        let prop_addr = read.word() as usize;
        self.ui.debug(&format!("Property table @ {:x}", prop_addr));

        let text_length = self.memory.read_byte(prop_addr);
        let short_name = if text_length > 0 {
            self.read_zstring(prop_addr + 1)
        } else {
            String::new()
        };

        self.ui.debug(&format!("{} (len: {})\n", short_name, text_length));
    }

    fn get_object_number(&self, input: &str) -> u16 {
        if let Ok(num) = input.parse() {
            num
        } else if let Some(num) = self.find_object(input) {
            num
        } else {
            0
        }
    }

    fn debug_object(&mut self, input: &str) {
        let num = self.get_object_number(input);
        if num == 0 {
            return;
        }

        let mut obj = Object::new(num, self);
        self.add_object_children(&mut obj);
        self.ui.debug(&obj.to_string());
    }

    fn debug_object_tree(&mut self) {
        let tree = self.get_object_tree();
        self.ui.debug(&tree.to_string());
    }

    fn debug_object_properties(&mut self, input: &str) {
        let num = self.get_object_number(input);
        if num == 0 {
            return;
        }

        self.ui.debug(&format!("Object: #{}", num));

        let addr = self.get_object_prop_table_addr(num);
        let str_length = self.memory.read_byte(addr) as usize * 2; // words in name
        let first_addr = addr + str_length + 1;

        let mut prop = self.read_object_prop(first_addr);
        let mut slice = self.memory.read(prop.addr, prop.len as usize);

        self.ui.debug(&format!("{:2} {:?}", prop.num, slice));

        while prop.num != 0 {
            prop = self.read_object_prop(prop.next);
            slice = self.memory.read(prop.addr, prop.len as usize);

            self.ui.debug(&format!("{:2} {:?}", prop.num, slice));
        }
    }

    fn debug_object_attributes(&mut self, input: &str) {
        let num = self.get_object_number(input);
        if num == 0 {
            return;
        }

        let name = self.get_object_name(num);
        let mut attributes = Vec::new();

        for i in 0..(self.attr_width * 8) as u16 {
            if self.test_attr(num, i) == 1 {
                attributes.push(i);
            }
        }

        self.ui.debug(&format!("{} ({})\n{:?}", name, num, attributes));
    }

    pub fn debug_object_details(&self, obj_num: u16) -> String {
        if obj_num == 0 {
            return String::new();
        }

        let mut out = String::from("Properties:\n");

        let addr = self.get_object_prop_table_addr(obj_num);
        let str_length = self.memory.read_byte(addr) as usize * 2; // words in name
        let first_addr = addr + str_length + 1;

        let mut prop = self.read_object_prop(first_addr);
        let mut slice = self.memory.read(prop.addr, prop.len as usize);

        writeln!(out, "{:2} {:?}", prop.num, slice).unwrap();

        while prop.num != 0 {
            prop = self.read_object_prop(prop.next);
            slice = self.memory.read(prop.addr, prop.len as usize);

            writeln!(out, "{:2} {:?}", prop.num, slice).unwrap();
        }

        let mut attributes = Vec::new();

        for i in 0..(self.attr_width * 8) as u16 {
            if self.test_attr(obj_num, i) == 1 {
                attributes.push(i);
            }
        }

        write!(out, "\nAttributes:\n{:?}", attributes).unwrap();

        out
    }

    fn debug_have_attribute(&mut self, attr_str: &str) {
        let attr = attr_str.parse().unwrap_or(0);
        let mut objects = Vec::new();

        for obj_num in 1..self.get_total_object_count() + 1 {
            if self.test_attr(obj_num, attr) == 1 {
                objects.push(Object::new(obj_num, self));
            }
        }

        for obj in &objects {
            self.ui.debug(&obj.to_string());
        }
    }

    fn debug_have_property(&mut self, prop_str: &str) {
        let prop_num = prop_str.parse().unwrap_or(0);
        let mut objects = Vec::new();

        for obj_num in 1..self.get_total_object_count() + 1 {
            let prop = self.find_prop(obj_num, prop_num);

            if prop.num != 0 {
                objects.push(Object::new(obj_num, self));
            }
        }

        for obj in &objects {
            self.ui.debug(&obj.to_string());
        }
    }

    fn debug_room(&mut self) {
        let mut room = Object::new(self.read_global(0), self);
        self.add_object_children(&mut room);
        self.ui.debug(&room.to_string());
    }

    fn debug_yourself(&mut self) {
        if let Some(num) = self.find_yourself() {
            let mut you = Object::new(num, self);
            self.add_object_children(&mut you);
            self.ui.debug(&you.to_string());
        }
    }

    fn debug_parent(&mut self, input: &str) {
        let num = self.get_object_number(input);
        if num == 0 {
            return;
        }

        let parent_num = self.get_parent(num);

        if parent_num == 0 {
            self.ui.debug(&"Parent if the root object 0");
        } else {
            let mut parent = Object::new(parent_num, self);
            self.add_object_children(&mut parent);
            self.ui.debug(&parent.to_string());
        }
    }

    fn debug_find_object(&mut self, name: &str) {
        if let Some(num) = self.find_object(name) {
            let object = Object::new(num, self);
            self.ui.debug(&format!("{}", object));
        }
    }

    fn debug_teleport(&mut self, input: &str) {
        let you = self.find_yourself().expect("Can't find you in the object tree");
        let num = self.get_object_number(input);

        if num == 0 {
            self.ui.print("I can't find that room...\n");
            return;
        } else {
            self.ui.print("Zzzap! Somehow you are in a different place...\n");
        }

        self.insert_obj(you, num);
    }

    fn debug_steal(&mut self, input: &str) {
        let you = self.find_yourself().expect("Can't find you in the object tree");
        let num = self.get_object_number(input);

        if num == 0 {
            self.ui.print("I can't find that object...\n");
            return;
        } else {
            self.ui.print(&format!("Zzzing! Somehow you are holding the {}...\n", input));
        }

        self.insert_obj(num, you);
    }

    pub fn debug_history(&mut self) {
        let undo_count = self.undos.len();
        let total = self.undos.len() + self.redos.len() + 1;

        self.ui.debug(&"History:");

        for (i, state) in self.undos.iter().enumerate() {
            let index = i + 1;
            self.ui.debug(&format!("    ({}/{}) @ {}", index, total, state.0));
        }

        if let Some(ref current) = self.current_state {
            let index = undo_count + 1;
            self.ui.debug(&format!(" -> ({}/{}) @ {}", index, total, current.0));
        }

        for (i, state) in self.redos.iter().rev().enumerate() {
            let index = undo_count + i + 2;
            self.ui.debug(&format!("    ({}/{}) @ {}", index, total, state.0));
        }
    }

    fn debug_routine(&mut self, routine_addr: usize) {
        let mut read = self.memory.get_reader(routine_addr);
        let mut locals = Vec::new();
        let count = read.byte();

        for _ in 0..count {
            match self.version {
                1..=4 => locals.push(read.word()),
                _ => locals.push(0),
            };
        }

        let first_instr = self.decode_instruction(read.position());
        let mut set: HashSet<Instruction> = HashSet::new();

        fn follow(zvm: &Zmachine, set: &mut HashSet<Instruction>, instr: Instruction) {
            if set.contains(&instr) {
                return;
            }

            let branch = match instr.branch {
                Some(Branch { address: Some(addr), .. }) => Some(addr),
                _ => None,
            };

            let next = if instr.advances() {
                Some(instr.next)
            } else {
                None
            };

            set.insert(instr);

            if let Some(addr) = branch {
                follow(zvm, set, zvm.decode_instruction(addr));
            };

            if let Some(addr) = next {
                follow(zvm, set, zvm.decode_instruction(addr));
            };
        }

        follow(self, &mut set, first_instr);

        let mut instructions = set.iter().collect::<Vec<_>>();
        instructions.sort_by_key(|i| i.addr);

        self.ui.debug(&format!("Locals: {:?}", locals));

        for instr in &instructions {
            self.ui.debug(&format!("{}", instr));
        }
    }
}

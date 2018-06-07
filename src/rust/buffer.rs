#![allow(dead_code)]


#[derive(Debug)]
pub struct Reader<'a> {
    buffer: &'a Buffer,
    cursor: usize,
}

impl<'a> Reader<'a> {
    pub fn byte(&mut self) -> u8 {
        let byte = self.buffer.read_byte(self.cursor);
        self.cursor += 1;
        byte
    }

    pub fn word(&mut self) -> u16 {
        let word = self.buffer.read_word(self.cursor);
        self.cursor += 2;
        word
    }

    pub fn position(&self) -> usize {
        self.cursor
    }

    pub fn seek(&mut self, addr: usize) {
        self.cursor = addr;
    }
}


#[derive(Debug)]
pub struct Writer<'a> {
    buffer: &'a mut Buffer,
    cursor: usize,
}

impl<'a> Writer<'a> {
    pub fn byte(&mut self, value: u8) {
        self.buffer.write_byte(self.cursor, value);
        self.cursor += 1;
    }

    pub fn word(&mut self, value: u16) {
        self.buffer.write_word(self.cursor, value);
        self.cursor += 2;
    }

    pub fn position(&self) -> usize {
        self.cursor
    }

    pub fn seek(&mut self, addr: usize) {
        self.cursor = addr;
    }
}


#[derive(Debug)]
pub struct Buffer {
    buf: Vec<u8>,
}

impl Buffer {
    pub fn new(buf: Vec<u8>) -> Buffer {
        Buffer { buf }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn read_byte(&self, location: usize) -> u8 {
        self.buf[location]
    }

    pub fn read_word(&self, location: usize) -> u16 {
        ((self.buf[location] as u16) << 8) + self.buf[location+1] as u16
    }

    pub fn write_byte(&mut self, location: usize, value: u8) {
        self.buf[location] = value;
    }

    pub fn write_word(&mut self, location: usize, value: u16) {
        let top = ((value & 0xFF00) >> 8) as u8;
        let bottom = (value & 0x00FF) as u8;

        self.buf[location] = top;
        self.buf[location+1] = bottom;
    }

    pub fn read(&self, location: usize, length: usize) -> &[u8] {
        &self.buf[location..location+length]
    }

    pub fn write(&mut self, location: usize, buf: &[u8]) {
        for (i, b) in buf.iter().enumerate() {
            self.write_byte(location + i as usize, *b);
        }
    }

    pub fn slice(&self, start: usize, end: usize) -> &[u8] {
        &self.buf[start..end]
    }

    pub fn get_reader(&self, cursor: usize) -> Reader {
        Reader { buffer: self, cursor }
    }

    pub fn get_writer(&mut self, cursor: usize) -> Writer {
        Writer { buffer: self, cursor }
    }
}

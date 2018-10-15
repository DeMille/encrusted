use frame::Frame;
use std::fmt;

#[derive(Debug)]
pub struct QuetzalSave {
    pub pc: usize,
    pub memory: Vec<u8>,
    pub frames: Vec<Frame>,
    pub chksum: u16,
}

impl QuetzalSave {
    fn empty() -> QuetzalSave {
        QuetzalSave {
            pc: 0,
            memory: Vec::new(),
            frames: Vec::new(),
            chksum: 0,
        }
    }

    pub fn from_bytes(save_data: &[u8], original_dynamic: &[u8]) -> QuetzalSave {
        let mut save = QuetzalSave::empty();

        let (form_header, _, form_body) = QuetzalSave::read_chunk(save_data);
        if form_header != "FORM" {
            panic!("Can't find FORM header, bad save file?")
        }

        let chunks = &form_body[4..]; // skip the IFZS string at the start
        let mut offset = 0;

        while offset < chunks.len() {
            let next = &chunks[offset..];
            let (header, length, body) = QuetzalSave::read_chunk(next);

            if header == "IFhd" {
                save.read_ifhd_body(body);
            } else if header == "Stks" {
                save.read_stks_body(body);
            } else if header == "CMem" {
                save.read_cmem_body(body, original_dynamic);
            } else if header == "UMem" {
                save.memory = body.to_vec();
            }

            // skip any other unnecessary chunks
            if save.is_complete() {
                break;
            }

            offset += length;
        }

        if !save.is_complete() {
            panic!("Save file doesn't contain all necesary fields");
        }

        save
    }

    pub fn make(
        pc: usize,
        current: &[u8],
        original: &[u8],
        frames: &[Frame],
        chksum: u16,
        release: u16,
        serial: &[u8],
    ) -> Vec<u8> {
        let mut save_data = Vec::new();
        let mut form_body = Vec::from(&b"IFZS"[..]); // Form starts w/ "IFZS"

        let ifhd_body = QuetzalSave::make_ifhd_body(release, serial, chksum, pc);
        let stks_body = QuetzalSave::make_stks_body(frames);
        let cmem_body = QuetzalSave::make_cmem_body(current, original);

        QuetzalSave::write_chunk(&mut form_body, "IFhd", &ifhd_body[..]);
        QuetzalSave::write_chunk(&mut form_body, "Stks", &stks_body[..]);
        QuetzalSave::write_chunk(&mut form_body, "CMem", &cmem_body[..]);
        QuetzalSave::write_chunk(&mut save_data, "FORM", &form_body[..]);

        save_data
    }

    fn read_chunk(data: &[u8]) -> (String, usize, &[u8]) {
        let header = String::from_utf8_lossy(&data[0..4]).into_owned();

        let mut body_length = 0;
        body_length += (data[4] as usize) << 24;
        body_length += (data[5] as usize) << 16;
        body_length += (data[6] as usize) << 8;
        body_length += data[7] as usize;

        let body = &data[8..(8 + body_length)];

        // chunks get padded with an empty 0 byte if they have an odd length
        let mut chunk_length = 8 + body_length;
        if chunk_length % 2 != 0 {
            chunk_length += 1;
        }

        (header, chunk_length, body)
    }

    fn write_chunk(bytes: &mut Vec<u8>, header: &str, body: &[u8]) {
        // 4 bytes for the header string
        bytes.extend(&header.as_bytes()[0..4]);

        // 4 bytes for the length (BE)
        let length = body.len();
        bytes.push(((length & 0xFF00_0000) >> 24) as u8);
        bytes.push(((length & 0x00FF_0000) >> 16) as u8);
        bytes.push(((length & 0x0000_FF00) >> 8) as u8);
        bytes.push((length & 0x0000_00FF) as u8);

        // + body
        bytes.extend(body);

        // If the body length is odd, add a padding byte. This extra byte is
        // *not* included in the length marker above.
        if body.len() % 2 != 0 {
            bytes.push(0);
        }
    }

    fn is_complete(&self) -> bool {
        self.pc != 0 && self.chksum != 0 && !self.frames.is_empty() && !self.memory.is_empty()
    }

    fn read_ifhd_body(&mut self, bytes: &[u8]) {
        // 1 word for release (skip, we don't use)
        // 6 bytes for serial number (also skip)

        // 1 word for checksum
        self.chksum += u16::from(bytes[8]) << 8;
        self.chksum += u16::from(bytes[9]);

        // 3 bytes for PC
        self.pc += usize::from(bytes[10]) << 16;
        self.pc += usize::from(bytes[11]) << 8;
        self.pc += usize::from(bytes[12]);
    }

    fn make_ifhd_body(release: u16, serial: &[u8], chksum: u16, pc: usize) -> [u8; 13] {
        let mut bytes = [0; 13]; // ifhd body is always 13 bytes

        // 1 word for release
        bytes[0] = ((release & 0xFF00) >> 8) as u8;
        bytes[1] = (release & 0x00FF) as u8;

        // 6 bytes for serial number
        bytes[2] = serial[0];
        bytes[3] = serial[1];
        bytes[4] = serial[2];
        bytes[5] = serial[3];
        bytes[6] = serial[4];
        bytes[7] = serial[5];

        // 1 word for checksum
        bytes[8] = ((chksum & 0xFF00) >> 8) as u8;
        bytes[9] = (chksum & 0x00FF) as u8;

        // 3 bytes for PC
        bytes[10] = ((pc & 0xFF_0000) >> 16) as u8;
        bytes[11] = ((pc & 0x00_FF00) >> 8) as u8;
        bytes[12] = (pc & 0x00_00FF) as u8;

        bytes
    }

    fn read_cmem_body(&mut self, compressed: &[u8], original: &[u8]) {
        let mut uncompressed = Vec::new();
        let mut index = 0;

        while index < compressed.len() {
            let byte = compressed[index];

            // non-zero bytes are bytes that are different that the orignal
            if byte != 0 {
                uncompressed.push(byte);
                index += 1;
            // zero bytes are followed by a length byte, indicating how many
            // 0s go between the previous non-zero byte (above) and the next
            } else {
                // +1 for the 0 before the length byte:
                let length = compressed[index + 1] as usize;
                uncompressed.extend(vec![0; length + 1]);
                index += 2;
            }
        }

        let difference = original.len() - uncompressed.len();

        if difference > 0 {
            uncompressed.extend(vec![0; difference]);
        }

        // XOR uncompressed with original to restore
        self.memory = uncompressed
            .iter()
            .zip(original.iter())
            .map(|(a, b)| a ^ b)
            .collect();
    }

    fn make_cmem_body(current: &[u8], original: &[u8]) -> Vec<u8> {
        // match each byte of the current and the original
        current.iter().zip(original.iter())
            // XOR current dynamic memory with the original (get what changed)
            .map(|(a, b)| a ^ b)
            // compress result by counting zeros instead of including them all
            // reduces over tuple of (compressed_bytes, current_zero_count)
            .fold((Vec::new(), 0), |(mut compressed, mut zero_count), byte| {
                // Non-zero bytes (byte differs between current and original)
                if byte != 0 {
                    // if there were any 0 bytes being counted, add them here
                    if zero_count > 0 {
                        compressed.push(0);
                        compressed.push(zero_count - 1); // # of 0s *after* the 1st
                    }
                    // add xor'd byte and reset 0 counter
                    compressed.push(byte);
                    zero_count = 0;
                // 1 byte = a max of 255 zeros we can count, so guard here
                } else if zero_count == 255 {
                    compressed.push(0);
                    compressed.push(zero_count);
                    zero_count = 0;
                // otherwise just increment the zero counter for 0 bytes
                } else {
                    zero_count += 1;
                }

                (compressed, zero_count)
            }).0 // <- compressed is the first field in the tuple
    }

    fn read_stks_body(&mut self, bytes: &[u8]) {
        let mut frames = Vec::new();
        let mut offset = 0;

        while offset < bytes.len() - 1 {
            // variable lengths found here:
            let num_locals = bytes[offset + 3] & 0b0000_1111;
            let mut stack_length = 0;
            stack_length += u16::from(bytes[offset + 6]) << 8;
            stack_length += u16::from(bytes[offset + 7]);

            // locals start @ byte 8, stack values start after locals
            // each value is a 2 byte word
            let end = offset + 8 + num_locals as usize * 2 + stack_length as usize * 2;

            let slice = &bytes[offset..end];
            let frame = Frame::from_bytes(slice);

            frames.push(frame);
            offset += slice.len();
        }

        self.frames = frames;
    }

    fn make_stks_body(frames: &[Frame]) -> Vec<u8> {
        let mut bytes = Vec::new();

        for frame in frames.iter() {
            bytes.extend(frame.to_vec());
        }

        bytes
    }
}

impl fmt::Display for QuetzalSave {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "PC: {:#04x} Chksum: {:#04x} Mem Length: {}",
            self.pc,
            self.chksum,
            self.memory.len()
        )?;

        for frame in &self.frames {
            writeln!(f, "{}", frame)?;
        }

        write!(f, "")
    }
}

#![allow(dead_code)]

use std::boxed::Box;
use std::io;
use std::io::Write;

use regex::Regex;
use term_size;

use traits::UI;

lazy_static! {
    static ref ANSI_RE: Regex = Regex::new(
        r"[\x1b\x9b][\[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-PRZcf-nqry=><]"
    ).unwrap();
}

#[derive(Debug)]
pub struct TerminalUI {
    width: usize,
    x_position: usize,
}

impl TerminalUI {
    fn print_raw(&self, raw: &str) {
        print!("{}", raw);
        io::stdout().flush().unwrap();
    }

    fn enter_alternate_screen(&self) {
        if self.is_term() {
            self.print_raw("\x1B[?1049h");
        }
    }

    fn end_alternate_screen(&self) {
        if self.is_term() {
            self.print_raw("\x1B[?1049l");
        }
    }

    fn is_term(&self) -> bool {
        self.width != 0
    }
}

impl UI for TerminalUI {
    fn new() -> Box<TerminalUI> {
        if let Some((w, _)) = term_size::dimensions() {
            Box::new(TerminalUI {
                width: w,
                x_position: 0,
            })
        } else {
            Box::new(TerminalUI {
                width: 0,
                x_position: 0,
            })
        }
    }

    fn clear(&self) {
        // Clear screen: ESC [2J
        // Move cursor to 1x1: [H
        if self.is_term() {
            self.print_raw("\x1B[2J\x1B[H");
        }
    }

    fn print(&mut self, text: &str) {
        if !self.is_term() {
            self.print_raw(text);
            return;
        }

        let lines: Vec<_> = text.lines().collect();
        let num_lines = lines.len();

        // implements some word-wrapping so words don't get split across lines
        lines.iter().enumerate().for_each(|(i, line)| {
            // skip if this line is just the result of a "\n"
            if !line.is_empty() {
                let words: Vec<_> = line.split_whitespace().collect();
                let num_words = words.len();

                // check that each word can fit on the line before printing it.
                // if its too big, bump to the next line and reset x-position
                words.iter().enumerate().for_each(|(i, word)| {
                    self.x_position += word.len();

                    if self.x_position > self.width {
                        self.x_position = word.len();
                        print!("\n");
                    }

                    print!("{}", word);

                    // add spaces back in if we can (an not on the last element)
                    if i < num_words - 1 && self.x_position < self.width {
                        self.x_position += 1;
                        print!(" ");
                    }
                });
            }

            // add newlines back that were removed from split
            if i < num_lines - 1 {
                print!("\n");
                self.x_position = 0;
            }
        });

        io::stdout().flush().unwrap();
    }

    fn debug(&mut self, text: &str) {
        self.print(text);
    }

    fn print_object(&mut self, object: &str) {
        if self.is_term() {
            self.print_raw("\x1B[37;1m");
        }
        self.print(object);
        if self.is_term() {
            self.print_raw("\x1B[0m");
        }
    }

    fn set_status_bar(&self, left: &str, right: &str) {
        // ESC ]2; "text" BEL
        if self.is_term() {
            self.print_raw(&format!("\x1B]2;{}  -  {}\x07", left, right));
        }
    }

    fn get_user_input(&self) -> String {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Error reading input");

        // trim, strip and control sequences that might have gotten in,
        // and then trim once more to get rid of any excess whitespace
        ANSI_RE
            .replace_all(input.trim(), "")
            .to_string()
            .trim()
            .to_string()
    }

    fn reset(&self) {
        println!("");
    }

    // unimplemented, only used in web ui
    fn flush(&mut self) {}
    fn message(&self, _mtype: &str, _msg: &str) {}
}

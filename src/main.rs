use std::collections::HashMap;
use std::env;
use std::fs;
use std::str;

// capture station data
#[derive(Debug)]
struct StationData {
    min: f64,
    max: f64,
    sum: f64,
    count: f64,
    avg: f64,
}

// define parser inputs
struct Parser<'a> {
    input: &'a str,
    index: usize,
}

// parser instantiation
impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Parser<'a> {
        Parser {
            input: input,
            index: 0,
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = (&'a [u8], f64);

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;

        // handle when index reaches end of length
        if index >= self.input.len() {
            return None;
        }

        unsafe {
            // turn input into byte iterator
            let input = self.input;
            let input_bytes = input.as_bytes();
            // if this is destroyed and recreated every time, it will only iterate through the first section,
            // meaning that it will incorrectly count for each subsequent line after the first
            let mut input_bytes_iter = input_bytes.iter();

            // create cursor
            let mut cursor_index = index;

            // track name start
            let mut name_start_index = index;

            // advance cursor_index with next_code_point until semicolon is reached
            // TODO: DRY
            loop {
                let (code_point, len_bytes) = next_code_point(&mut input_bytes_iter).unwrap();
                if code_point == ';' as u32 {
                    break;
                }
                cursor_index += len_bytes;
            }

            // cursor_index should now be at ';'
            debug("semicolon", input_bytes, cursor_index, cursor_index + 1);

            // get name
            let found_name = input_bytes.get_unchecked(name_start_index..cursor_index);

            cursor_index += 1; // cursor is now on first digit of temp

            // track temp start
            let temp_start_index = cursor_index;

            // TODO: DRY
            loop {
                let (code_point, len_bytes) = next_code_point(&mut input_bytes_iter).unwrap();
                if code_point == '\n' as u32 {
                    break;
                }
                cursor_index += len_bytes;
            }

            // at this point, cursor_index == index of '\n' on last line
            debug("newln", input_bytes, cursor_index, cursor_index + 1);

            // TODO: consolidate loop code with reused code below in some way
            let found_temp_str = input_bytes.get_unchecked(temp_start_index..cursor_index);
            debug("temp", input_bytes, temp_start_index, cursor_index);

            let found_number = str::from_utf8(found_temp_str)
                .unwrap()
                .parse()
                .expect("thought this was a f64");

            if (cursor_index < self.input.len()) {
                cursor_index += 1;
            }
            debug("line advance", input_bytes, 0, cursor_index);

            // catch up iterator index with cursor
            self.index = cursor_index;

            return Some((found_name, found_number));
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    read(file_path);
}

fn read(arg: &str) {
    // read in file
    let contents = fs::read_to_string(arg).expect("cannot read file");
    let mut parsing_machine: Parser = Parser::new(&contents);
    for (label, value) in parsing_machine {
        println!("");
    }
}

// INLINE FUNCTIONS --------

// debug probe
#[inline]
unsafe fn debug(label: &str, input_bytes: &[u8], start_index: usize, cursor_index: usize) {
    let found_temp_str = input_bytes.get_unchecked(start_index..cursor_index);
    println!("{}: {:?}", label, str::from_utf8(found_temp_str).unwrap());
}

// utf-8 initial byte handler
// inlining removes the need for a function call
#[inline]
const fn utf8_first_byte(byte: u8, width: u32) -> u32 {
    (byte & (0x7F >> width)) as u32
}

// utf-8 continuing byte accumulator
#[inline]
const fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 {
    // create 6 trailing spaces, then strip header and accumulate via bitwise OR
    (ch << 6) | (byte & 0x3F) as u32
}

// hard-code value for continuing byte mask
const CONT_MASK: u8 = 0b0011_1111;

// returns (code_point, len_bytes)
#[inline]
pub unsafe fn next_code_point<'a, I: Iterator<Item = &'a u8>>(
    bytes: &mut I,
) -> Option<(u32, usize)> {
    // handle ASCII if header byte is in range
    let x = *bytes.next()?;
    let mut len_bytes = 1;
    if x < 128 {
        return Some((x as u32, len_bytes));
    }

    len_bytes += 1;

    // [[[x y] z] w] case
    // NOTE: Performance is sensitive to the exact formulation here
    let init = utf8_first_byte(x, 2);
    // SAFETY: `bytes` produces an UTF-8-like string,
    // so the iterator must produce a value here.
    let y = unsafe { *bytes.next().unwrap_unchecked() };
    let mut ch = utf8_acc_cont_byte(init, y);
    if x >= 0xE0 {
        len_bytes += 1;
        // [[x y z] w] case
        // 5th bit in 0xE0 .. 0xEF is always clear, so `init` is still valid
        // SAFETY: `bytes` produces an UTF-8-like string,
        // so the iterator must produce a value here.
        let z = unsafe { *bytes.next().unwrap_unchecked() };
        let y_z = utf8_acc_cont_byte((y & CONT_MASK) as u32, z);
        ch = init << 12 | y_z;
        if x >= 0xF0 {
            len_bytes += 1;
            // [x y z w] case
            // use only the lower 3 bits of `init`
            // SAFETY: `bytes` produces an UTF-8-like string,
            // so the iterator must produce a value here.
            let w = unsafe { *bytes.next().unwrap_unchecked() };
            ch = (init & 7) << 18 | utf8_acc_cont_byte(y_z, w);
        }
    }

    Some((ch, len_bytes))
}

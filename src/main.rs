use hashbrown::HashMap;
use hashbrown::hash_map::Entry;
use rustc_hash::FxBuildHasher;
use std::env;
use std::fs;
use std::str;
use std::str::from_utf8_unchecked;
use std::time::Instant;

// capture station data
#[derive(Debug)]
struct StationData {
    min: i16,
    max: i16,
    sum: i64,
    count: u32,
}

// define parser inputs
struct Parser<'a> {
    input: &'a str,
    input_iter: std::slice::Iter<'a, u8>,
    index: usize,
}

// parser instantiation
impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Parser<'a> {
        Parser {
            input: input,
            input_iter: input.as_bytes().iter(),
            index: 0,
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = (&'a [u8], i16);

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;

        // handle when index reaches end of length
        if index >= self.input.len() {
            return None;
        }

        unsafe {
            let input_bytes = self.input.as_bytes();
            let mut input_bytes_iter = &mut self.input_iter;

            // create cursor
            let mut cursor_index = index;

            // track name start
            let name_start_index = index;

            // advance cursor_index with next_code_point until semicolon is reached
            // TODO: DRY
            loop {
                let byte = next_byte(input_bytes_iter).unwrap();
                if byte == ';' as u8 {
                    break;
                }
                cursor_index += 1;
            }

            // cursor_index should now be at ';'
            // debug("semicolon", input_bytes, cursor_index, cursor_index + 1);

            // get name
            let found_name = input_bytes.get_unchecked(name_start_index..cursor_index);

            cursor_index += 1; // cursor is now on first digit of temp

            // track temp start
            let temp_start_index = cursor_index;

            // TODO: DRY
            loop {
                let byte = next_byte(input_bytes_iter).unwrap();
                if byte == '\n' as u8 {
                    break;
                }
                cursor_index += 1;
            }

            // at this point, cursor_index == index of '\n' on last line
            // debug("newln", input_bytes, cursor_index, cursor_index + 1);

            // TODO: consolidate loop code with reused code below in some way
            let found_temp_bytes = input_bytes.get_unchecked(temp_start_index..cursor_index);
            // debug("temp", input_bytes, temp_start_index, cursor_index);

            let found_number: i16 = parse_temp(found_temp_bytes);

            if (cursor_index < self.input.len()) {
                cursor_index += 1;
            }
            // debug("line advance", input_bytes, 0, cursor_index);

            // catch up iterator index with cursor
            self.index = cursor_index;

            return Some((found_name, found_number));
        }
    }
}

fn main() {
    let now = Instant::now();
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    read(file_path);
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}

fn read(arg: &str) {
    let contents = fs::read_to_string(arg).expect("cannot read file");

    let mut places: HashMap<&[u8], StationData, FxBuildHasher> =
        HashMap::with_capacity_and_hasher(1024, FxBuildHasher::default());
    let parsing_machine: Parser = Parser::new(&contents);

    for (label, value) in parsing_machine {
        // check and update hashmap
        match places.entry(label) {
            Entry::Occupied(mut current_station) => {
                let current_data = current_station.get_mut();
                if value > current_data.max {
                    current_data.max = value;
                }
                if value < current_data.min {
                    current_data.min = value;
                }
                current_data.count += 1;
                current_data.sum += value as i64;
            }
            Entry::Vacant(empty) => {
                empty.insert(StationData {
                    min: value,
                    max: value,
                    sum: value as i64,
                    count: 1,
                });
            }
        }
    }

    // sort keys
    let mut sorted_places: Vec<&&[u8]> = places.keys().collect();
    sorted_places.sort();

    // output to console
    print!("{{");
    for (i, key) in sorted_places.iter().enumerate() {
        let city_name = unsafe { from_utf8_unchecked(key) };
        let city_data = &places[**key];
        let city_avg = (city_data.sum as f64 / city_data.count as f64) / 10.0;

        if i > 0 {
            print!(", ");
        }

        print!(
            "{}={:.1}/{:.1}/{:.1}",
            city_name,
            city_data.min as f64 / 10.0,
            city_avg,
            city_data.max as f64 / 10.0
        );
    }
    println!("}}");
}

// INLINE FUNCTIONS --------

#[inline]
unsafe fn debug(label: &str, input_bytes: &[u8], start_index: usize, cursor_index: usize) {
    let found_temp_str = unsafe { input_bytes.get_unchecked(start_index..cursor_index) };
    println!("{}: {:?}", label, str::from_utf8(found_temp_str).unwrap());
}

#[inline]
fn parse_temp(bytes: &[u8]) -> i16 {
    let (neg, rem_bytes) = if bytes[0] == b'-' {
        (true, &bytes[1..])
    } else {
        (false, bytes)
    };
    let num: i16 = match rem_bytes {
        [ones, b'.', deci] => (ones - b'0') as i16 * 10 + (deci - b'0') as i16,
        [tens, ones, b'.', deci] => {
            (tens - b'0') as i16 * 100 + (ones - b'0') as i16 * 10 + (deci - b'0') as i16
        }
        _ => panic!("err {:?}", rem_bytes),
    };
    if neg { num * -1 } else { num }
}

// #[inline]
// fn adv_cursor() {
//     loop {
//         let byte = next_byte(input_bytes_iter).unwrap();
//         if byte == ';' as u8 {
//             break;
//         }
//         cursor_index += 1;
//     }
// }

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

#[inline]
pub fn next_byte<'a, I: Iterator<Item = &'a u8>>(bytes: &mut I) -> Option<u8> {
    let byte = *bytes.next()?;
    Some(byte)
}

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

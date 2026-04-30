use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::env;
use std::fs;

#[derive(Debug)]
struct StationData {
    min: f64,
    max: f64,
    sum: f64,
    count: f64,
}

struct Parser<'a> {
    input: &'a str,
    loc: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Parser<'a> {
        Parser {
            input: input,
            loc: 0,
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = (&'a str, f64);

    fn next(&mut self) -> Option<Self::Item> {
        let loc = self.loc;

        // handle when loc reaches end of length
        if loc == self.input.len() {
            return None;
        }
        // pub unsafe fn next_code_point<'a, I: Iterator<Item = &'a u8>>(bytes: &mut I) -> Option<u32> {
        unsafe {
            // get input and convert to bytes
            let input = self.input;
            let mut input_bytes = input.as_bytes().iter();

            // after the first char because names' length >= 1
            let mut new_loc = loc + 1;

            // check letter by letter for semicolon
            while next_code_point(&mut input_bytes).unwrap() != ';' as u32 {
                new_loc += 1;
            }

            // at this point, new_loc == loc of ';'
            let found_name = input.get_unchecked(loc..new_loc);

            new_loc += 1;
            // at this point, new_loc == loc after ';'

            let f64_start_loc = new_loc;

            // check current char if newline
            while next_code_point(&mut input_bytes).unwrap() != '\n' as u32 {
                new_loc += 1;
            }

            // at this point, new_loc == loc of '\n'
            let found_stat_string = input.get_unchecked(f64_start_loc..new_loc);

            new_loc += 1;
            // at this point, new_loc == loc after '\n'
            self.loc = new_loc;

            // i16, range -99.9, 99.9
            let found_number = found_stat_string.parse().expect("thought this was a f64");

            return Some((found_name, found_number));
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    read_back(file_path);
}

fn read_back(arg: &str) {
    // read in file
    let contents = fs::read_to_string(arg).expect("y no read");

    // get max, min, sum, count
    let list = format(&contents);

    // get example station
    let example_name = "Şuḩār";
    let data = &list[example_name];

    // get example average
    let avg = (data.sum / data.count * 10.0).round() / 10.0;
    // iterate on HashMap in-place
    // for (_key, data) in list.iter_mut() {
    //     data.avg = (data.sum / data.count * 10.0).round() / 10.0;
    // }

    // print example station
    println!("{:#?},{:#?},{:#?}", avg, data.min, data.max);
}

fn format<'a>(arg: &'a String) -> HashMap<&'a str, StationData> {
    // optimization ideas
    // call custom parser to advance line by line through the file which returns label and value, rather than relying on default functions
    // &str is arbitrary length, if we can set max length based on string length then we can optimize
    // remove exception handling if know all data is well-formed

    // create empty hashmap
    let mut places: HashMap<&str, StationData> = HashMap::new();

    let data = arg.split('\n');

    // iterate through all data points
    for point in data {
        // split creates another copy/reference
        let mut data_pair = point.split(';');

        // operate on iterator
        let label = data_pair.next().unwrap_or("unknown");
        let value: f64 = data_pair.next().unwrap_or("0").parse().unwrap_or(0.0);

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
                current_data.count += 1.0;
                current_data.sum += value;
            }
            Entry::Vacant(empty) => {
                let new_data = StationData {
                    min: value,
                    max: value,
                    // avg: 0.0,
                    sum: value,
                    count: 1.0,
                };
                empty.insert(new_data);
            }
        }
    }
    places
}

// utf-8 initial byte handler
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
pub unsafe fn next_code_point<'a, I: Iterator<Item = &'a u8>>(bytes: &mut I) -> Option<u32> {
    // handle ASCII if header byte is in range
    let x = *bytes.next()?;
    if x < 128 {
        return Some(x as u32);
    }

    // [[[x y] z] w] case
    // NOTE: Performance is sensitive to the exact formulation here
    let init = utf8_first_byte(x, 2);
    // SAFETY: `bytes` produces an UTF-8-like string,
    // so the iterator must produce a value here.
    let y = unsafe { *bytes.next().unwrap_unchecked() };
    let mut ch = utf8_acc_cont_byte(init, y);
    if x >= 0xE0 {
        // [[x y z] w] case
        // 5th bit in 0xE0 .. 0xEF is always clear, so `init` is still valid
        // SAFETY: `bytes` produces an UTF-8-like string,
        // so the iterator must produce a value here.
        let z = unsafe { *bytes.next().unwrap_unchecked() };
        let y_z = utf8_acc_cont_byte((y & CONT_MASK) as u32, z);
        ch = init << 12 | y_z;
        if x >= 0xF0 {
            // [x y z w] case
            // use only the lower 3 bits of `init`
            // SAFETY: `bytes` produces an UTF-8-like string,
            // so the iterator must produce a value here.
            let w = unsafe { *bytes.next().unwrap_unchecked() };
            ch = (init & 7) << 18 | utf8_acc_cont_byte(y_z, w);
        }
    }

    Some(ch)
}

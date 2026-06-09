use hashbrown::HashMap;
use hashbrown::hash_map::Entry;
use memmap2::MmapOptions;
use rustc_hash::FxBuildHasher;
use std::env;
use std::fs::File;
use std::str::from_utf8_unchecked;
use std::thread;
use std::time::Instant;

// capture station data
#[derive(Debug, Default)]
struct StationData {
    min: i16,
    max: i16,
    sum: i64,
    count: u32,
}

// define parser inputs
struct Parser<'a> {
    input: &'a [u8],
    input_iter: std::slice::Iter<'a, u8>,
    index: usize,
}

// parser instantiation
impl<'a> Parser<'a> {
    fn new(input: &'a [u8]) -> Parser<'a> {
        Parser {
            input: input,
            input_iter: input.iter(),
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
            let input_bytes = self.input;
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

fn read(
    map: &[u8],
) -> Result<HashMap<&[u8], StationData, FxBuildHasher>, Box<dyn std::error::Error>> {
    let mut places: HashMap<&[u8], StationData, FxBuildHasher> =
        HashMap::with_capacity_and_hasher(1024, FxBuildHasher::default());
    let parsing_machine: Parser = Parser::new(map);

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

    Ok(places)
}

fn main() {
    let now = Instant::now();
    let args: Vec<String> = env::args().collect();

    // memory map file
    let file_path = &args[1];
    let file = File::open(file_path).unwrap();
    let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };

    let threads = 9;
    let length = mmap.len();
    let split = length / threads;

    // house maps from threads
    let mut threaded_maps: Vec<HashMap<&[u8], StationData, FxBuildHasher>> = (0..threads)
        .map(|_| HashMap::with_hasher(FxBuildHasher::default()))
        .collect();

    // turn into mutable iterator
    // to get each individual hashmap
    let mut map_iterator = threaded_maps.iter_mut();

    // chunk file and map to threads
    thread::scope(|s| {
        let mut start = 0;
        for i in 0..threads {
            // have last chunk end at EOF
            let mut end = if i == threads - 1 {
                length
            } else {
                start + split
            };

            // find nearest newline
            while end < length && mmap[end] != b'\n' {
                end += 1;
            }

            // include newline
            if end < length {
                end += 1;
            }

            // define chunk
            let chunk = &mmap[start..end];

            let map_ref = map_iterator.next().unwrap();

            // spawn thread
            s.spawn(move || {
                *map_ref = read(chunk).unwrap();
            });

            start = end;
        }
    });

    // combine hashmaps
    let merged_map = threaded_maps.into_iter().fold(
        HashMap::with_hasher(FxBuildHasher::default()),
        |mut acc: HashMap<&[u8], StationData, FxBuildHasher>,
         map: HashMap<&[u8], StationData, FxBuildHasher>| {
            for (k, v) in map {
                let acc_stn = acc.entry(k).or_default();

                acc_stn.count += v.count;
                acc_stn.sum += v.sum;

                if v.max > acc_stn.max {
                    acc_stn.max = v.max;
                }
                if v.min < acc_stn.min {
                    acc_stn.min = v.min;
                }
            }
            acc
        },
    );

    // parse file
    // let places = read(&mmap).unwrap();

    // sort keys
    let mut sorted_places: Vec<&&[u8]> = merged_map.keys().collect();
    sorted_places.sort();

    // output to console
    print!("{{");
    for (i, key) in sorted_places.iter().enumerate() {
        let city_name = unsafe { from_utf8_unchecked(key) };
        let city_data = &merged_map[**key];
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

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}

// INLINE FUNCTIONS --------

// #[inline]
// unsafe fn debug(label: &str, input_bytes: &[u8], start_index: usize, cursor_index: usize) {
//     let found_temp_str = unsafe { input_bytes.get_unchecked(start_index..cursor_index) };
//     println!("{}: {:?}", label, str::from_utf8(found_temp_str).unwrap());
// }

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

#[inline]
pub fn next_byte<'a, I: Iterator<Item = &'a u8>>(bytes: &mut I) -> Option<u8> {
    let byte = *bytes.next()?;
    Some(byte)
}

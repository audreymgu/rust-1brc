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

impl StationData {
    fn new(temp: i16) -> Self {
        Self {
            max: temp,
            min: temp,
            sum: temp as i64,
            count: 1,
        }
    }

    fn update(&mut self, temp: i16) {
        self.max = i16::max(self.max, temp);
        self.min = i16::min(self.min, temp);
        self.sum += temp as i64;
        self.count += 1;
    }
}

// define parser inputs
struct Parser<'a> {
    input: &'a [u8],
    index: usize,
}

// parser instantiation
impl<'a> Parser<'a> {
    fn new(input: &'a [u8]) -> Parser<'a> {
        Parser {
            input: input,
            index: 0,
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = (&'a [u8], i16);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut end = self.index;

        // handle when index reaches end of length
        if self.index >= self.input.len() {
            return None;
        }

        unsafe {
            let input = self.input;

            while input[end] != b';' {
                end += 1;
            }

            end += 1;

            let found_name = input.get_unchecked(self.index..end);

            self.index = end;

            while input[end] != b'\n' {
                end += 1;
            }

            let found_temp_bytes = input.get_unchecked(self.index..end);

            let found_temp: i16 = parse_temp(found_temp_bytes);

            self.index = end;

            if self.index < self.input.len() {
                self.index += 1;
            }

            return Some((found_name, found_temp));
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
        // match places.entry(label) {
        //     Entry::Occupied(mut current_station) => {
        //         let current_data = current_station.get_mut();
        //         if value > current_data.max {
        //             current_data.max = value;
        //         }
        //         if value < current_data.min {
        //             current_data.min = value;
        //         }
        //         current_data.count += 1;
        //         current_data.sum += value as i64;
        //     }
        //     Entry::Vacant(empty) => {
        //         empty.insert(StationData {
        //             min: value,
        //             max: value,
        //             sum: value as i64,
        //             count: 1,
        //         });
        //     }
        // }
        places
            .entry(label)
            .and_modify(|prev: &mut StationData| prev.update(value))
            .or_insert(StationData::new(value));
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

    let threads = 8;
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

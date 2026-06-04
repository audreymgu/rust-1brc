use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::env;
use std::fs;
use std::time::Instant;

#[derive(Debug)]
struct StationData {
    min: i32,
    max: i32,
    avg: f64,
    sum: i64,
    count: i64,
}

fn main() {
    let now = Instant::now();
    let args: Vec<String> = env::args().collect();
    // TODO: handle when arg not received
    let file_path = &args[1];
    read_back(file_path);
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}

fn read_back(arg: &str) {
    let contents = fs::read_to_string(arg).expect("y no read");

    let mut list = format(&contents);
    // iterate on HashMap in-place
    for (_key, data) in list.iter_mut() {
        let float_sum = data.sum as f64 / 10.0;
        let float_count = data.count as f64;
        data.avg = (float_sum / float_count);
    }
    println!("{:#?}", list);
}

fn format<'a>(arg: &'a String) -> HashMap<&'a str, StationData> {
    // create empty hashmap
    let mut places: HashMap<&str, StationData> = HashMap::new();

    // get each data point
    // optimization: remove .collect(), use iterator directly
    // saves approx 1 sec when benchmarked
    // swap with lines()
    let data = arg.lines();

    // iterate through all data points
    for point in data {
        // split by semicolon and collect
        // optimization: use split_once rather than split
        let mut data_pair = point.split_once(';').unwrap();

        // operate on iterator
        let label = data_pair.0;
        let float_value: f64 = data_pair.1.parse().unwrap();
        let int_value: i32 = (float_value * 10.0) as i32;

        // check and update hashmap
        match places.entry(label) {
            Entry::Occupied(mut current_station) => {
                let current_data = current_station.get_mut();
                if int_value > current_data.max {
                    current_data.max = int_value;
                }
                if int_value < current_data.min {
                    current_data.min = int_value;
                }
                current_data.count += 1;
                current_data.sum += int_value as i64;
            }
            Entry::Vacant(empty) => {
                let new_data = StationData {
                    min: int_value,
                    max: int_value,
                    avg: 0.0,
                    sum: int_value as i64,
                    count: 1,
                };
                empty.insert(new_data);
            }
        }
    }
    places
}

// pseudo-code
// get file contents DONE
// split by new-line > Vec DONE
// for item in vec
// check name list
// if not in list, create Vec to store value (should this just be a reference?)
// for each name list
// calculate min?
// hold one value in memory and compare, switch out when smaller
// calculate max?
// hold one value in memory and compare, switch out when larger
// calculate average
// just... get the average?

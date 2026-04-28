use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::env;
use std::fs;

#[derive(Debug)]
struct StationData {
    min: f64,
    max: f64,
    avg: f64,
    sum: f64,
    count: f64,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    // TODO: handle when arg not received
    let file_path = &args[1];
    read_back(file_path);
}

fn read_back(arg: &str) {
    let contents = fs::read_to_string(arg).expect("y no read");

    let mut list = format(&contents);
    for (_key, data) in list.iter_mut() {
        data.avg = (data.sum / data.count * 10.0).round() / 10.0;
    }
    println!("{:#?}", list);
}

fn format<'a>(arg: &'a String) -> HashMap<&'a str, StationData> {
    // create empty hashmap
    let mut places: HashMap<&str, StationData> = HashMap::new();

    // get each data point
    let data: Vec<&str> = arg.split('\n').collect();

    // iterate through all data points
    for point in data.iter() {
        // split by semicolon and collect
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
                    avg: 0.0,
                    sum: value,
                    count: 1.0,
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

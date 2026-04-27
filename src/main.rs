use std::env;
use std::fs;

fn main() {
    let args:Vec<String> = env::args().collect();
    // TODO: handle when arg not received
    let file_path = &args[1];
    read_back(file_path);
}

fn read_back(arg: &str) {
    // println!("Your file path is {arg}");
    
    let contents = fs::read_to_string(arg)
        .expect("y no read");

    let list = format(&contents);
    println!("{:#?}", list);
}

fn format<'a>(arg: &'a String) -> Vec<&'a str> {
    let mut places: Vec<&str> = Vec::new();
    let data: Vec<&str> = arg.split('\n').collect();
    // {:#?} is an option for pretty-printed debug output
    // println!("{:#?}", data);
    for &point in data.iter() {
        for &place in places.iter() {
            if point.contains(place) {
                continue;
            } else {
                let new_place: &str = point.split(';')
                    .next()
                    .unwrap_or("error");
                places.push(new_place);
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

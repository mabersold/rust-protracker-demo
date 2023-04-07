use std::env;
use std::path::Path;
use std::fs::File;
use std::io::Read;

struct ProTrackerModule {
    title: String
}

fn main() {
    // Get the path to the file from the command line argument
    let args: Vec<String> = env::args().collect();
    if args.len() == 0 {
        println!("You must include a path to the file you wish to load.");
        std::process::exit(1);
    }

    let file_path = &args[1];
    let path = Path::new(file_path);
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut buffer = [0u8; 20];
    file.read_exact(&mut buffer).expect("Failed to read title");
    let title = String::from_utf8_lossy(&buffer).trim_end().to_string();

    let protracker_module = ProTrackerModule {
        title
    };

    println!("Title: {}", protracker_module.title);
}

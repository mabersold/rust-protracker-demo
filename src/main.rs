use std::env;
use std::path::Path;
use std::fs::File;
use std::io::Read;

struct ProTrackerModule {
    title: String,
    instruments: Vec<Instrument>
}

struct Instrument {
    name: String,
    length: u16,
    fine_tune: i8,
    volume: u8,
    repeat_offset: u16,
    repeat_length: u16
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

    // Read the title
    let mut buffer = [0u8; 20];
    file.read_exact(&mut buffer).expect("Failed to read title");
    let title = String::from_utf8_lossy(&buffer).trim_end().to_string();

    // Read the instruments
    let mut instruments = Vec::with_capacity(31);
    for _ in 0..31 {
        let mut instrument_name_buf = [0u8; 22];
        file.read_exact(&mut instrument_name_buf).expect("Failed to read instrument name");
        let instrument_name = String::from_utf8_lossy(&instrument_name_buf).trim_end().to_string();

        let mut instrument_length_buf = [0u8; 2];
        file.read_exact(&mut instrument_length_buf).expect("Failed to read instrument length");
        let instrument_length = u16::from_le_bytes(instrument_length_buf);

        let mut instrument_fine_tune_buf = [0u8; 1];
        file.read_exact(&mut instrument_fine_tune_buf).expect("Failed to read instrument finetune");
        let instrument_fine_tune = signed_nibble((instrument_fine_tune_buf[0] & 0x0F) as i8);

        let mut instrument_volume_buf = [0u8; 1];
        file.read_exact(&mut instrument_volume_buf).expect("Failed to read instrument volume");
        let instrument_volume = u8::from_le_bytes(instrument_volume_buf);

        let mut instrument_repeat_offset_buf = [0u8; 2];
        file.read_exact(&mut instrument_repeat_offset_buf).expect("Failed to read instrument repeat offset");
        let instrument_repeat_offset = u16::from_le_bytes(instrument_repeat_offset_buf);

        let mut instrument_repeat_length_buf = [0u8; 2];
        file.read_exact(&mut instrument_repeat_length_buf).expect("Failed to read instrument repeat length");
        let instrument_repeat_length = u16::from_le_bytes(instrument_repeat_length_buf);

        let instrument = Instrument {
            name: instrument_name,
            length: instrument_length,
            fine_tune: instrument_fine_tune,
            volume: instrument_volume,
            repeat_offset: instrument_repeat_offset,
            repeat_length: instrument_repeat_length
        };

        instruments.push(instrument);
    }

    let protracker_module = ProTrackerModule {
        title,
        instruments
    };

    println!("Title: {}", protracker_module.title);
    for instrument in protracker_module.instruments {
        println!("Instrument: {}", instrument.name);
        println!("Length: {}", instrument.length);
        println!("Fine tune: {}", instrument.fine_tune);
        println!("Volume: {}", instrument.volume);
        println!("Repeat offset: {}", instrument.repeat_offset);
        println!("Repeat length: {}", instrument.repeat_length);
    }
}

fn signed_nibble(data: i8) -> i8 {
    // Get rid of the first 4 bits
    let nibble = data & 15;

    //if first bit is 1, it's a negative number
    if nibble & 8 == 8 {
        (nibble - 16)
    } else {
        nibble
    }
}

use std::env;
use std::path::Path;
use std::fs::File;
use std::io::Read;

struct ProTrackerModule {
    title: String,
    instruments: Vec<Instrument>,
    total_song_positions: u8,
    noise_tracker_restart: u8,
    order_list: Vec<u8>,
    patterns: Vec<Pattern>
}

struct Instrument {
    name: String,
    length: u16,
    fine_tune: i8,
    volume: u8,
    repeat_offset: u16,
    repeat_length: u16,
    audio_data: Vec<u8>
}

struct Pattern {
    channels: Vec<Channel>
}

struct Channel {
    rows: Vec<Row>
}

struct Row {
    instrument_number: u8,
    pitch: u16,
    effect: EffectType,
    effect_x_value: u8,
    effect_y_value: u8
}

#[derive(Debug)]
enum EffectType {
    None,
    Arpeggio,
    PitchSlideUp,
    PitchSlideDown,
    SlideToNote,
    Vibrato,
    SlideToNoteWithVolumeSlide,
    VibratoWithVolumeSlide,
    VolumeSlide,
    InstrumentOffset,
    SetVolume,
    PatternBreak,
    FineVolumeSlideUp,
    FineVolumeSlideDown,
    ChangeSpeed,
    SetFineTune,
    PositionJump,
    UnknownEffect
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

    let title = read_string(20, &mut file, "Failed to read title");

    let mut instrument_names = Vec::with_capacity(31);
    let mut instrument_lengths = [0u16; 31];
    let mut instrument_fine_tunes = [0i8; 31];
    let mut instrument_volumes = [0u8; 31];
    let mut instrument_repeat_offsets = [0u16; 31];
    let mut instrument_repeat_lengths = [0u16; 31];

    // Read the instruments
    let mut instruments = Vec::with_capacity(31);
    for i in 0..31 {
        let instrument_name = read_string(22, &mut file, "Failed to read instrument name");
        instrument_names.push(instrument_name);

        let mut instrument_length_buf = [0u8; 2];
        file.read_exact(&mut instrument_length_buf).expect("Failed to read instrument length");
        let instrument_length = u16::from_be_bytes(instrument_length_buf);
        instrument_lengths[i] = instrument_length;

        let mut instrument_fine_tune_buf = [0u8; 1];
        file.read_exact(&mut instrument_fine_tune_buf).expect("Failed to read instrument finetune");
        let instrument_fine_tune = signed_nibble((instrument_fine_tune_buf[0] & 0x0F) as i8);
        instrument_fine_tunes[i] = instrument_fine_tune;

        let mut instrument_volume_buf = [0u8; 1];
        file.read_exact(&mut instrument_volume_buf).expect("Failed to read instrument volume");
        let instrument_volume = u8::from_be_bytes(instrument_volume_buf);
        instrument_volumes[i] = instrument_volume;

        let mut instrument_repeat_offset_buf = [0u8; 2];
        file.read_exact(&mut instrument_repeat_offset_buf).expect("Failed to read instrument repeat offset");
        let instrument_repeat_offset = u16::from_be_bytes(instrument_repeat_offset_buf);
        instrument_repeat_offsets[i] = instrument_repeat_offset;

        let mut instrument_repeat_length_buf = [0u8; 2];
        file.read_exact(&mut instrument_repeat_length_buf).expect("Failed to read instrument repeat length");
        let instrument_repeat_length = u16::from_be_bytes(instrument_repeat_length_buf);
        instrument_repeat_lengths[i] = instrument_repeat_length;
    }

    let mut total_positions_buf = [0u8; 1];
    file.read_exact(&mut total_positions_buf).expect("Failed to read number of song positions");
    let total_song_positions = u8::from_le_bytes(total_positions_buf);

    let mut noise_tracker_restart_buf = [0u8; 1];
    file.read_exact(&mut noise_tracker_restart_buf).expect("Failed to read Noise Tracker restart value");
    let noise_tracker_restart = u8::from_le_bytes(noise_tracker_restart_buf);

    let mut order_list = Vec::with_capacity(128);
    for _ in 0..128 {
        let mut order_number_buf = [0u8; 1];
        file.read_exact(&mut order_number_buf).expect("Failed to read an order number");
        let order = u8::from_le_bytes(order_number_buf);
        order_list.push(order);
    }

    let signature = read_string(4, &mut file, "Failed to read signature");

    let max_pattern_number = order_list.iter().max().unwrap();

    let mut patterns = Vec::new();
    for pattern_number in 0..*max_pattern_number {
        // println!("Pattern: {}", pattern_number);

        let mut channels = Vec::new();
        for i in 0..4 {
            channels.insert(i, Channel { rows: Vec::new() });
        }

        let mut pattern = Pattern { channels };

        for row_number in 0..64 {
            // println!("Row: {}", row_number);
            for channel_number in 0..4 {
                let mut row_buffer = [0u8; 4];
                file.read_exact(&mut row_buffer).expect("Failed to read file");

                let instrument_number = ((row_buffer[0] & 0b1111) << 4) | (row_buffer[2] >> 4);
                let pitch = ((row_buffer[0] & 0b1111) as u16) << 8 | row_buffer[1] as u16;
                let effect_number: u8 = row_buffer[2] & 0b1111;
                let effect_x_value: u8 = row_buffer[3] >> 4;
                let effect_y_value: u8 = row_buffer[3] & 0b00001111;

                let effect = get_effect(effect_number, effect_x_value, effect_y_value);

                let row = Row {
                    instrument_number,
                    pitch,
                    effect,
                    effect_x_value,
                    effect_y_value
                };

                pattern.channels[channel_number].rows.push(row);
            }
        }

        patterns.push(pattern);
    }

    let mut audio_data_vectors = Vec::with_capacity(31);
    for length in instrument_lengths {
        let audio_data_size = length as usize * 2;
        let mut audio_data_buffer = vec![0u8; audio_data_size];
        file.read_exact(&mut audio_data_buffer).expect("Could not load audio data");
        audio_data_vectors.push(audio_data_buffer);
    }

    for i in 0..31 {
        let instrument = Instrument {
            name: instrument_names[i].clone(),
            length: instrument_lengths[i],
            fine_tune: instrument_fine_tunes[i],
            volume: instrument_volumes[i],
            repeat_offset: instrument_repeat_offsets[i],
            repeat_length: instrument_repeat_lengths[i],
            audio_data: audio_data_vectors[i].clone()
        };
        instruments.push(instrument);
    }

    let protracker_module = ProTrackerModule {
        title,
        instruments,
        total_song_positions,
        noise_tracker_restart,
        order_list,
        patterns
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
    println!("Total number of song positions: {}", protracker_module.total_song_positions);
    println!("Noise Tracker restart value: {}", protracker_module.noise_tracker_restart);
    print!("Order list: ");
    for order in protracker_module.order_list {
        print!("{} ", order);
    }
    println!("\nSignature: {}", signature);
    for (pattern_number, pattern) in protracker_module.patterns.iter().enumerate() {
        println!("Pattern {}", pattern_number);
        for (channel_number, channel) in pattern.channels.iter().enumerate() {
            println!("Channel {}", channel_number);
            for (row_number, row) in channel.rows.iter().enumerate() {
                println!("Row: {}, Instrument: {}, pitch: {}, effect: {:?}, x: {}, y: {}", row_number, row.instrument_number, row.pitch, row.effect, row.effect_x_value, row.effect_y_value);
            }
        }
    }
}

fn read_string(length: usize, file: &mut File, error_message: &str) -> String {
    let mut buffer = [0u8; 256];
    file.read_exact(&mut buffer[..length]).expect(error_message);
    String::from_utf8_lossy(&buffer[..length]).trim_end().to_string()
}

fn signed_nibble(data: i8) -> i8 {
    // Get rid of the first 4 bits
    let nibble = data & 15;

    //if first bit is 1, it's a negative number
    if nibble & 8 == 8 {
        nibble - 16
    } else {
        nibble
    }
}

fn get_effect(effect: u8, x_value: u8, y_value: u8) -> EffectType {
    match effect {
        0 => {
            if x_value == 0 && y_value == 0 {
                EffectType::None
            } else {
                EffectType::Arpeggio
            }
        },
        1 => EffectType::PitchSlideUp,
        2 => EffectType::PitchSlideDown,
        3 => EffectType::SlideToNote,
        4 => EffectType::Vibrato,
        5 => EffectType::SlideToNoteWithVolumeSlide,
        6 => EffectType::VibratoWithVolumeSlide,
        9 => EffectType::InstrumentOffset,
        10 => EffectType::VolumeSlide,
        11 => EffectType::PositionJump,
        12 => EffectType::SetVolume,
        13 => EffectType::PatternBreak,
        14 => {
            match x_value {
                5 => EffectType::SetFineTune,
                10 => EffectType::FineVolumeSlideUp,
                11 => EffectType::FineVolumeSlideDown,
                _ => EffectType::UnknownEffect
            }
        }
        15 => EffectType::ChangeSpeed,
        _ => EffectType::UnknownEffect
    }
}

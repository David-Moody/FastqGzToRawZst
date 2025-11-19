use flate2::read::MultiGzDecoder;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process;
use std::time::Instant;
use zstd::stream::{AutoFinishEncoder, Encoder};
// use kdam::tqdm;

pub struct Config {
    pub input_filename: String,
    pub output_filename: String,
}
const MEBIBYTE: usize = 2_usize.pow(20);
const BUFFER_SIZE: usize = 2 * MEBIBYTE;
const ZSTD_COMPRESSION_LEVEL: i32 = 3;

#[inline(always)]
pub fn write_line_to_buffer(line: &str, buffer: &mut impl Write) {
    buffer
        .write(line.as_bytes())
        .expect("Failed to write data to file");
    // Add the newline. Might be more efficient to do this in memory?
    // Newline not required as we no longer strip lines
    //    buffer.write(b"\n").expect("Failed to write data to file");
}
pub fn get_buffer_to_output_file(filename: &str) -> BufWriter<File> {
    let f_out_headers = File::create(filename).expect("Unable to create file");
    let f_out_headers: BufWriter<File> = BufWriter::with_capacity(BUFFER_SIZE, f_out_headers);
    f_out_headers
}
fn get_read_buffer_to_gzipped_file(config: Config) -> BufReader<MultiGzDecoder<BufReader<File>>> {
    let f_in = File::open(config.input_filename).expect("Unable to open file");
    let f_in: BufReader<_> = BufReader::with_capacity(BUFFER_SIZE, f_in);
    let f_in = MultiGzDecoder::new(f_in);
    let f_in = BufReader::with_capacity(BUFFER_SIZE, f_in);

    return f_in;
}

fn parse_file_singlethread(mut f_in: impl BufRead, output_filename: &str) {
    // Create ZSTD output buffer to compress on the fly
    let output_buffer: BufWriter<File> = get_buffer_to_output_file(output_filename);
    let mut output_buffer_zstd: Encoder<BufWriter<File>> =
        Encoder::new(output_buffer, ZSTD_COMPRESSION_LEVEL).unwrap();
    output_buffer_zstd
        .multithread(1)
        .expect("Missing zstdmt feature?");
    let mut output_buffer_zstd: AutoFinishEncoder<'_, BufWriter<File>> =
        output_buffer_zstd.auto_finish();

    let mut index_mod: u8 = 2;
    let mut reused_line: String = String::new();
    while f_in.read_line(&mut reused_line).unwrap() != 0 {
        index_mod += 1;
        if index_mod == 4 {
            write_line_to_buffer(&reused_line, &mut output_buffer_zstd);
            index_mod = 0;
        }
        reused_line.clear();
    }
    // match index_mod {
    //     0 => {
    //         // Headers
    //     }
    //     1 => {
    //         // Reads
    //         write_line_to_buffer(&line, &mut output_buffer_zstd);
    //     }
    //     2 => {
    //         // Comments
    //     }
    //     3 => {
    //         // Qualities
    //     }
    //     _ => panic!("Remainder should never be anything outside 0,1,2,3"),
    // }
    //}
}

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        println!("Problem parsing argument: {err}");
        process::exit(1);
    });
    println!("Processing {}", config.input_filename);

    let start_time = Instant::now();

    let output_filename = config.output_filename.clone();
    let f_in = get_read_buffer_to_gzipped_file(config);
    parse_file_singlethread(f_in, &output_filename);

    let time_duration = start_time.elapsed().as_secs();
    println!("{:?}s", time_duration);
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();

        let input_filename: String = match args.next() {
            Some(arg) => arg,
            None => return Err("No input filepath given"),
        };
        let output_filename: String = match args.next() {
            Some(arg) => arg,
            None => return Err("No output filepath given"),
        };

        // Avoiding needing the command line arg when rapidly developing

        // let input_filename = String::from("data/1_control_ITS2_2019_minq7.fastq");
        return Ok(Config {
            input_filename,
            output_filename,
        });
    }
}

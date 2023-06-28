extern crate flate2;

use chrono::{DateTime, Utc};
use flate2::read::GzDecoder;
use nom::bytes::complete::tag;
use nom::character::complete::multispace0;
use nom::combinator::opt;
use nom::sequence::{delimited, tuple};
use nom_json_parser::{parser, IResult, Json, VerboseError};
use std::io;
use std::io::prelude::*;

fn main() {
    let start_time: DateTime<Utc> = Utc::now();

    let stdin = io::stdin();
    let stdin = stdin.lock();

    let d = GzDecoder::new(stdin);

    // Accumulate and drain from this
    let mut buffer = String::new();

    // Buffer for single chunk reads
    const CHUNK_BUFFER_SIZE: usize = 32768;
    let mut chunk_buffer: [u8; CHUNK_BUFFER_SIZE] = [0; CHUNK_BUFFER_SIZE];

    // The decompressed stream
    let mut reader = io::BufReader::new(d);

    let mut count = 0;

    while let Ok(size) = reader.read(&mut chunk_buffer) {
        // successfully reading to the EOF means we can stop
        if size == 0 {
            break;
        }

        // Grab a string from the chunk buffer as utf8
        let chunk_str = std::str::from_utf8(&chunk_buffer).unwrap();

        // println!("Chunk  => [>{}<] ({} bytes)", chunk_str, size);

        // Accumulate to the buffer
        buffer.push_str(&chunk_str[..size]);

        // println!("Buffer => [>{}<]", buffer);

        // Can we parse out a json packet yet?
        while let Ok((remaining, json)) = chunked_json_array_parser()(&buffer) {
            // How much did we eat?
            let processed_bytes = buffer.len() - remaining.len();

            // println!("Grab   => [>{}<] ({})", json, processed_bytes);

            // Print it to stdout!
            println!("{}", json);

            // Drain the parsed bytes
            buffer.drain(..processed_bytes);

            // Perf report
            count = count + 1;
            if count % 10000 == 0 {
                let mark_time: DateTime<Utc> = Utc::now();
                let elapsed_time = mark_time.signed_duration_since(start_time);
                eprintln!(
                    "Processed {} records ({:.2} seconds elapsed, avg {:.2} records per second)",
                    count,
                    (elapsed_time.num_milliseconds() as f64 / 1000.0),
                    (count as f64 / elapsed_time.num_milliseconds() as f64) * 1000.0,
                );
            }
        }

        if buffer.len() > CHUNK_BUFFER_SIZE * 10 {
            println!("Fail.");
            break;
        }
    }
}

// The secret sauce - a custom json parser, which extracts a single element at a time
// from within the json array, rather than loading the entire document into memory
//
pub type Parser<T, U> = IResult<T, U, VerboseError<T>>;
fn chunked_json_array_parser() -> impl Fn(&str) -> Parser<&str, Json> {
    move |i| {
        delimited(
            multispace0,
            tuple((
                opt(tag("[")),
                opt(tag(",")),
                delimited(multispace0, parser, multispace0),
                opt(tag("]")),
            )),
            multispace0,
        )(i)
        .map(|(next, (_, _, v, _))| (next, v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_parser() {
        let input = "[{\"p1\":\"1\"},{\"p2\":\"1\"}]";
        let d = chunked_json_array_parser()(input);
        println!("Input => {:?}", d);

        let input = " [ {\"p1\":\"1\"},{\"p2\":\"1\"}]";
        let x = chunked_json_array_parser()(input);
        println!("Input => {:?}", x);
    }
}

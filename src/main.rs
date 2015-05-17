#![feature(rand, core, collections, convert, path_ext)]
extern crate rand;
extern crate core;
extern crate collections;
extern crate getopts;
extern crate image;

use std::env;
use std::vec::Vec;
use std::fs::{OpenOptions, PathExt};
use std::path::{Path, PathBuf};
use std::io::{Stdin, Read, stdin, Stdout, Write, stdout, stderr};
use core::num::Float;
use core::cmp::min;
use rand::{Rand, Rng};
use rand::isaac::IsaacRng;
use getopts::Options;
use image::{GenericImage, Pixel, DynamicImage};

const OUT_CHANNEL_COUNT: u8 = 4;
const DEFAULT_BITMASK_SIZE: u8 = 3;
const HEADER_SZ: usize = 20;



macro_rules! println_stderr(
    // from: https://stackoverflow.com/questions/27588416/how-to-send-output-to-stderr
    ($($arg:tt)*) => (
        match writeln!(&mut stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
        )
    );

macro_rules! print_stderr(
    // from: https://stackoverflow.com/questions/27588416/how-to-send-output-to-stderr
    ($($arg:tt)*) => (
        match write!(&mut stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
        )
    );


fn calc_image_capacity(img: &DynamicImage, mask_bitsize: u8) -> usize {
    assert!( mask_bitsize > 0 );
    assert!( mask_bitsize <= 8 );
    let (width, height) = img.dimensions();
    ((width * height * OUT_CHANNEL_COUNT as u32) as f64 * ((mask_bitsize as f64) / 8.0)).floor() as usize
}


///////////////////////////////////////////////////////////////////////
fn read_bitindex_from(vec: &Vec<u8>, chunk_bitindex: u64) -> u8 {
    // The following are equivalent to division/remainder by 8.
    // The compiler would probably end up generating this, anyway.
    let subbyte_idx = (chunk_bitindex >> 3) as usize;
    let subbit_idx = (chunk_bitindex & 0x000000000000000F) as usize;
    return (vec[subbyte_idx] & (1 << subbit_idx)) >> subbit_idx;
}


fn piggyback_data(prng: &mut IsaacRng, data_source: &mut Read, mask_bitsize: u8,
                  in_path: &Path, out_path: &Path) -> usize
{
    assert!( ! PathExt::exists(out_path) );
    let img = image::open(in_path).unwrap();
    let (width, height) = img.dimensions();
    let mut new_img = DynamicImage::new_rgba8(width, height);

    let total_capacity = calc_image_capacity(&img, mask_bitsize);
    assert!(total_capacity >= HEADER_SZ);
    let capacity = total_capacity - HEADER_SZ;

    let mut chunk_source = data_source.take(capacity as u64);
    let mut chunk_data_vec = Vec::<u8>::new();
    let chunk_bytes_read = match chunk_source.read_to_end(&mut chunk_data_vec) {
        Ok(chunk_bytes_read) => { chunk_bytes_read }
        Err(f)               => { panic!(f.to_string()) }
    };
    if chunk_bytes_read < 1 {
        return 0;
    }

    let mut header_vec: Vec<u8> = chunk_data_vec.len().to_string().into();
    assert!( header_vec.len() <= (HEADER_SZ as usize) );
    for _ in header_vec.len()..(HEADER_SZ as usize) {
        header_vec.push(' ' as u8);
    }
    let mut chunk_vec = header_vec.clone();
    chunk_vec.append(&mut chunk_data_vec);
    assert!( chunk_vec.len() <= (total_capacity as usize) );

    let chunk_max_bitindex = (chunk_vec.len() * 8) as u64;
    let mut chunk_bitindex = 0u64;

    for (x, y, pixel) in img.pixels() {
        let mut new_pixel = pixel.clone().to_rgba();;
        for channel in new_pixel.channels_mut() {
            let mut data = u8::rand::<IsaacRng>(prng);
            let data_bits_left = min(mask_bitsize as u64,
                                     chunk_max_bitindex - chunk_bitindex) as u8;
            for data_bitindex in 0..data_bits_left {
                data &= 0xFF ^ (1 << data_bitindex);
                data |= read_bitindex_from(&chunk_vec, chunk_bitindex) << data_bitindex;
                chunk_bitindex += 1;
            }
            let mask = ((1u16 << (mask_bitsize as u16)) - 1u16) as u8;
            let inv_mask = 0xFF ^ mask;
            *channel &= inv_mask;
            *channel |= mask & data;
        }
        new_img.put_pixel(x, y, new_pixel);
    }
    assert!( chunk_bitindex == chunk_max_bitindex );

    let ref mut fout = match
        OpenOptions::new()
        .read(false)
        .write(true)
        .create(true)
        .truncate(false)
        .open(out_path)
    {
            Ok(fout) => { fout }
            Err(e)   => { panic!(
                    "can't open file for output: {} ({:?})",
                    e, out_path) }
    };

    let _ = new_img.save(fout, image::PNG);
    chunk_bytes_read
}


fn encode(mask_bitsize: u8, image_paths_str: Vec<String>, output_dir_path_str: String) {
    let mut prng = IsaacRng::new_unseeded(); // FIXME: very bad
    let mut data_source: Stdin = stdin();
    let mut is_there_data = true;
    let mut img_idx = 0;
    while is_there_data {
        let mut shuffled_image_paths = image_paths_str.clone();
        prng.shuffle( shuffled_image_paths.as_mut_slice() );
        for in_path_str in shuffled_image_paths.clone() {
            let in_path = PathBuf::from(&in_path_str);
            let mut out_path = PathBuf::from(&output_dir_path_str);
            out_path.push( format!("img_{:020}.png", img_idx) );

            let data_bytes_read = piggyback_data(&mut prng, &mut data_source, mask_bitsize,
                                                 &in_path, &out_path);
            is_there_data = data_bytes_read > 0;
            if !is_there_data {
                break;
            }

            println_stderr!(
                "{:?}\n\t=> {:?}:\n\t{} bytes encoded\n",
                in_path, out_path, data_bytes_read);
            img_idx += 1;
        }
    }
}


///////////////////////////////////////////////////////////////////////
fn write_bitindex_to(vec: &mut Vec<u8>, chunk_bitindex: u64, bit: u8) {
    assert!( bit < 2 );
    // The following are equivalent to division/remainder by 8.
    // The compiler would probably end up generating this, anyway.
    let subbyte_idx = (chunk_bitindex >> 3) as usize;
    let subbit_idx = (chunk_bitindex & 0x000000000000000F) as usize;
    vec[subbyte_idx] |= !(0xFFu8 ^ (bit << subbit_idx));
}


fn unpiggyback_data(data_sink: &mut Write, mask_bitsize: u8, in_path: &Path) -> usize {
    let img = image::open(in_path).unwrap();

    let total_capacity = calc_image_capacity(&img, mask_bitsize);
    assert!(total_capacity >= HEADER_SZ);

    let mut chunk_vec = Vec::<u8>::with_capacity(total_capacity as usize);
    for _ in 0..total_capacity { chunk_vec.push(0u8); }

    let chunk_max_bitindex = (chunk_vec.len() * 8) as u64;
    let mut chunk_bitindex = 0u64;

    for (_x, _y, pixel) in img.pixels() {
        for channel in pixel.channels() {
            let data = *channel;
            let data_bits_left = min(mask_bitsize as u64,
                                     chunk_max_bitindex - chunk_bitindex) as u8;
            for data_bitindex in 0..data_bits_left {
                let bit = (data & (1 << data_bitindex)) >> data_bitindex;
                write_bitindex_to(&mut chunk_vec, chunk_bitindex, bit);
                chunk_bitindex += 1;
            }
        }
    }
    assert!( chunk_bitindex == chunk_max_bitindex );

    let (header_arr, chunk_data_arr) = chunk_vec.split_at(HEADER_SZ);
    let mut header_vec : Vec<u8> = Vec::new();
    header_vec.push_all(header_arr);
    let header_str = match String::from_utf8(header_vec) {
        Ok(header_str) => { header_str }
        Err(e)         => { panic!("not a valid header str; {}", e) }
    };
    let data_len = match header_str.trim().parse::<usize>() {
        Ok(data_len) => { data_len }
        Err(e)       => { panic!("not a valid len; {}", e) }
    };

    let mut chunk_data_vec: Vec<u8> = Vec::new();
    chunk_data_vec.push_all(chunk_data_arr);
    chunk_data_vec.truncate(data_len);
    let _ = data_sink.write(chunk_data_vec.as_slice());
    data_len
}


fn decode(mask_bitsize: u8, image_paths_str: Vec<String>) {
    let mut data_sink: Stdout = stdout();
    let mut in_paths : Vec<PathBuf> = Vec::new();
    for in_path_str in image_paths_str {
        in_paths.push( PathBuf::from(&in_path_str) );
    }
    in_paths.as_mut_slice().sort_by(|a, b| a.file_name().cmp( &(b.file_name()) ));

    for in_path in in_paths {
        let data_bytes_read = unpiggyback_data(&mut data_sink, mask_bitsize, &in_path);
        println_stderr!("{:?}:\n\t{} bytes decoded\n", in_path, data_bytes_read)
    }
}


///////////////////////////////////////////////////////////////////////
fn print_usage(program: &str, opts: Options) {
    // from getopts example
    let brief = format!("Usage: {} [options] [image1 [image2 ..", program);
    print_stderr!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optopt("e", "encode", "encode images and put them in PATH", "PATH");
    opts.optopt("b", "bitmask_size", "size (in bits) of the blending mask", "");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let image_paths_arr = if !matches.free.is_empty() {
        (&matches.free[0..]).clone()
    } else {
        print_usage(&program, opts);
        return;
    };
    let mut image_paths : Vec<String> = Vec::new();
    image_paths.push_all(image_paths_arr);

    let bitmask_size = match matches.opt_str("b") {
        None                   => { DEFAULT_BITMASK_SIZE }
        Some(bitmask_size_str) => {
            match bitmask_size_str.parse::<u8>() {
                Ok(bitmask_size) => { bitmask_size }
                Err(_)           => {
                    print_usage(&program, opts);
                    return
                }
            }
        }
    };

    if matches.opt_present("e") {
        match matches.opt_str("e") {
            Some(output_dir_path) => { encode(bitmask_size, image_paths, output_dir_path) }
            None => {
                print_usage(&program, opts);
                return;
            }
        };
    }
    else {
        decode(bitmask_size, image_paths);
    }
}

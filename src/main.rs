extern crate memreader;

use std::io::Read;
use std::iter::Enumerate;
use std::slice::Iter;
use std::thread;
use std::time::SystemTime;

use memreader::{MemReader, ProvidesSlices};
use sysinfo::{ProcessExt, RefreshKind, System, SystemExt};
use ws::{listen, Message};

// const PIXEL_FORMAT: &str = "gray8";
const WIDTH: usize = 1872;
const HEIGHT: usize = 1404;
const BYTES_PER_PIXEL: usize = 1;
const WINDOW_BYTES: usize = WIDTH * HEIGHT * BYTES_PER_PIXEL;

fn main() {
    println!("Listening...");
    listen("0.0.0.0:4444", |out| {
        println!("Windowsize: {}", WINDOW_BYTES);

        let pid = get_pid().unwrap();

        println!("Pid: {}", pid);

        let fb0_addr = get_fb0addr(pid).unwrap();

        println!("FB0: 0x{:X}", fb0_addr);
        println!("FB0: {}", fb0_addr);

        let reader = MemReader::new(pid as u32).unwrap();

        let mut buf_a = [0u8; WINDOW_BYTES];
        let mut buf_b = [0u8; WINDOW_BYTES];

        let mut use_buffer_a = true;

        println!("Before Thread");
        thread::spawn(move || {
            println!("In Thread");
            loop {
                let start_begin = SystemTime::now();

                if use_buffer_a {
                    fill_buffer(fb0_addr, &reader, &mut buf_a)
                } else {
                    fill_buffer(fb0_addr, &reader, &mut buf_b)
                }.unwrap();
                let read_time = start_begin.elapsed().unwrap().as_millis();

                let start = SystemTime::now();
                let equal = check_equality(&buf_a, &buf_b);
                let cmp_time = start.elapsed().unwrap().as_millis();
                print!("Equality: {}  ", equal);
                let start = SystemTime::now();
                if !equal {
                    let encoded = if use_buffer_a {
                        encode(&buf_a, WIDTH)
                    } else {
                        encode(&buf_b, WIDTH)
                    };

                    out.send(Message::Binary(encoded)).unwrap();
                }

                let enc_time = start.elapsed().unwrap().as_millis();

                // println!("Read: {}", check_equality(&buf_a, &buf_b));

                use_buffer_a = !use_buffer_a;
                println!("Read: {:>3} ms, Cmp: {:>3} ms, Enc: {:>3} ms, All: {:>3} ms", read_time, cmp_time, enc_time, start_begin.elapsed().unwrap().as_millis());
            }
        });
        move |msg| {
            println!("Got: {}", msg);
            Ok(())
        }
    }).unwrap();
    println!("Exiting!");

    // println!("{} bytes at location {} in process {}'s memory", WINDOW_BYTES, fb0_addr, pid);
}

fn check_equality(buf_a: &[u8], buf_b: &[u8]) -> bool {
    for i in 0..WINDOW_BYTES {
        if buf_a[i] != buf_b[i] {
            println!("Unequal: {}", i);
            return false;
        }
    }
    return true;
}

fn fill_buffer(fb0_addr: usize, reader: &MemReader, buf: &mut [u8]) -> std::io::Result<()> {
    reader.address_slice_len(fb0_addr, WINDOW_BYTES).read_exact(buf)
}

fn get_pixel_frequency(buf: &mut Vec<u8>) {
    let mut pixels = [0usize; 256];
    for x in buf {
        pixels[*x as usize] += 1;
    }

    for (value, count) in pixels.iter().enumerate() {
        if *count > 0 {
            if *count > 1024 * 1024 {
                println!("{:>3} has {:>3}.{:03}.{:03} mB", value, count / (1024 * 1024), (count % (1024 * 1024)) / 1024, count % (1024));
            } else if *count > 1024 {
                println!("{:>3} has     {:>3}.{:03} kB", value, count / 1024, count % 1024);
            } else {
                println!("{:>3} has         {:>3} bytes", value, count);
            }
        }
    }
}

fn get_pid() -> Option<i32> {
    System::new_with_specifics(RefreshKind::new().with_processes()).get_process_by_name("xochitl").iter()
        .map(|process| process.pid())
        .next()
}

fn get_fb0addr(pid: i32) -> Option<usize> {
    proc_maps::get_process_maps(pid).unwrap().iter()
        .filter(|item| if let Some(name) = item.filename() {
            name == "/dev/fb0"
        } else { false })
        .map(|item| {
            println!("{:?}", item);
            item.size() + item.start()
        })
        .next()
}

fn encode(original: &[u8], width: usize) -> Vec<u8> {
    let mut result = Vec::new();

    let mut iter = original.iter().enumerate();

    while let Some((i, pixel)) = iter.next() {
        let h = i / width;
        let w = i % width;

        if *pixel < 255 {
            let mut vec = encode_pixel_row(h as u16, w as u16, &mut iter);
            result.append(&mut vec);
        }
    }

    result
}

fn encode_pixel_row(start_h: u16, start_w: u16, iter: &mut Enumerate<Iter<u8>>) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::new();
    data.push((start_h >> 8) as u8);
    data.push((start_h & 255) as u8);
    data.push((start_w >> 8) as u8);
    data.push((start_w & 255) as u8);
    while let Some((_, pixel)) = iter.next() {
        if *pixel == 255 { break; }
        data.push(*pixel);
    }
    data.push(255);

    data
}

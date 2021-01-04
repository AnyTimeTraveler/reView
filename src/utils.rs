use crate::WINDOW_BYTES;
use memreader::{MemReader, ProvidesSlices};
use std::io::Read;
use sysinfo::{RefreshKind, System, SystemExt, ProcessExt};
use std::slice::Iter;
use std::iter::Enumerate;

pub(crate) fn check_equality(buf_a: &[u8], buf_b: &[u8]) -> bool {
    for i in 0..WINDOW_BYTES {
        if buf_a[i] != buf_b[i] {
            println!("Unequal: {}", i);
            return false;
        }
    }
    return true;
}

pub(crate) fn fill_buffer(fb0_addr: usize, reader: &MemReader, buf: &mut [u8]) -> std::io::Result<()> {
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

pub(crate) fn get_pid() -> Option<i32> {
    System::new_with_specifics(RefreshKind::new().with_processes()).get_process_by_name("xochitl").iter()
        .map(|process| process.pid())
        .next()
}

pub(crate) fn get_fb0addr(pid: i32) -> Option<usize> {
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

pub(crate) fn encode(original: &[u8], width: usize) -> Vec<u8> {
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

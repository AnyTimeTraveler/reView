extern crate memreader;

use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};

use memreader::{MemReader, ProvidesSlices};
use sysinfo::{ProcessExt, RefreshKind, System, SystemExt};

fn main() -> anyhow::Result<()> {
    const PIXEL_FORMAT: &str = "gray8";
    const WIDTH: usize = 1872;
    const HEIGHT: usize = 1404;
    const BYTES_PER_PIXEL: usize = 1;
    const WINDOW_BYTES: usize = WIDTH * HEIGHT * BYTES_PER_PIXEL;

    println!("Windowsize: {}", WINDOW_BYTES);

    let pid = get_pid().ok_or(anyhow::Error::msg("Could not get pid"))?;

    println!("Pid: {}", pid);

    let fb0_addr = get_fb0addr(pid)?;

    println!("FB0: 0x{:X}", fb0_addr);
    println!("FB0: {}", fb0_addr);


    let reader = MemReader::new(pid as u32).unwrap();

    let mut buf = vec![0; WINDOW_BYTES];

    reader.address_slice_len(fb0_addr, WINDOW_BYTES).read_exact(&mut buf).unwrap();

    println!("{} bytes at location {} in process {}'s memory", WINDOW_BYTES, fb0_addr, pid);

    // print!("Open...");
    // let mut mem = OpenOptions::new()
    //     .read(true)
    //     .open(format!("/proc/{}/mem", pid))
    //     .unwrap();
    // println!("OK");
    //
    // print!("Seek...");
    // mem.seek(SeekFrom::Start(fb0_addr.0 as u64)).unwrap();
    // println!("OK");
    //
    // let mut buf = [0u8; WINDOW_BYTES];
    //
    // print!("Read...");
    // mem.read_exact(&mut buf).unwrap();
    // println!("OK");

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open("/home/root/still.raw")
        .unwrap();

    file.write_all(&buf).unwrap();

    Ok(())
}

fn get_pid() -> Option<i32> {
    System::new_with_specifics(RefreshKind::new().with_processes()).get_processes().iter()
        .filter(|(_, process)| process.name() == "xochitl")
        .map(|(pid, _)| pid)
        .next()
        .map(|pid| *pid)
}

fn get_fb0addr(pid: i32) -> anyhow::Result<usize> {
    proc_maps::get_process_maps(pid)?.iter()
        .filter(|item| if let Some(name) = item.filename() {
            name == "/dev/fb0"
        } else { false })
        .map(|item| {
            println!("{:?}", item);
            item.size() + item.start()
        })
        .next()
        .ok_or(anyhow::Error::msg("Memmap not found"))
}

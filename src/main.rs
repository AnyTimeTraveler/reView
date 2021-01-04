mod utils;

use futures_util::future::{select, Either};
use futures_util::{SinkExt, StreamExt};
use log::*;
use std::net::SocketAddr;
use std::time::{Duration, SystemTime};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Error};
use tungstenite::{Message, Result};
use memreader::MemReader;

// const PIXEL_FORMAT: &str = "gray8";
pub const WIDTH: usize = 1872;
pub const HEIGHT: usize = 1404;
pub const BYTES_PER_PIXEL: usize = 1;
pub const WINDOW_BYTES: usize = WIDTH * HEIGHT * BYTES_PER_PIXEL;

async fn accept_connection(peer: SocketAddr, stream: TcpStream) {
    if let Err(e) = handle_connection(peer, stream).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}

async fn handle_connection(peer: SocketAddr, stream: TcpStream) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    info!("New WebSocket connection: {}", peer);
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut interval = tokio::time::interval(Duration::from_millis(1000));

    // Echo incoming WebSocket messages and send a message periodically every second.

    let mut msg_fut = ws_receiver.next();
    let mut tick_fut = interval.next();
    loop {
        match select(msg_fut, tick_fut).await {
            Either::Left((msg, tick_fut_continue)) => {
                match msg {
                    Some(msg) => {
                        let msg = msg?;
                        if msg.is_text() || msg.is_binary() {
                            ws_sender.send(msg).await?;
                        } else if msg.is_close() {
                            break;
                        }
                        tick_fut = tick_fut_continue; // Continue waiting for tick.
                        msg_fut = ws_receiver.next(); // Receive next WebSocket message.
                    }
                    None => break, // WebSocket stream terminated.
                };
            }
            Either::Right((_, msg_fut_continue)) => {
                println!("In Thread");
                println!("Windowsize: {}", WINDOW_BYTES);

                let pid = utils::get_pid().unwrap();

                println!("Pid: {}", pid);

                let fb0_addr = utils::get_fb0addr(pid).unwrap();

                println!("FB0: 0x{:X}", fb0_addr);
                println!("FB0: {}", fb0_addr);

                let reader = MemReader::new(pid as u32).unwrap();

                let mut buf_a = [0u8; WINDOW_BYTES];
                let mut buf_b = [0u8; WINDOW_BYTES];

                let mut use_buffer_a = true;

                loop {
                    let start_begin = SystemTime::now();

                    if use_buffer_a {
                        utils::fill_buffer(fb0_addr, &reader, &mut buf_a)
                    } else {
                        utils::fill_buffer(fb0_addr, &reader, &mut buf_b)
                    }.unwrap();
                    let read_time = start_begin.elapsed().unwrap().as_millis();

                    let start = SystemTime::now();
                    let equal = utils::check_equality(&buf_a, &buf_b);
                    let cmp_time = start.elapsed().unwrap().as_millis();
                    print!("Equality: {}  ", equal);
                    let start = SystemTime::now();
                    if !equal {
                        let encoded = if use_buffer_a {
                            utils::encode(&buf_a, WIDTH)
                        } else {
                            utils::encode(&buf_b, WIDTH)
                        };

                        ws_sender.send(Message::Binary(encoded)).await.unwrap();
                    }

                    let enc_time = start.elapsed().unwrap().as_millis();

                    // println!("Read: {}", check_equality(&buf_a, &buf_b));

                    use_buffer_a = !use_buffer_a;
                    println!("Read: {:>3} ms, Cmp: {:>3} ms, Enc: {:>3} ms, All: {:>3} ms", read_time, cmp_time, enc_time, start_begin.elapsed().unwrap().as_millis());
                }
                // ws_sender.send(Message::Binary(vec![])).await?;
                msg_fut = msg_fut_continue; // Continue receiving the WebSocket message.
                tick_fut = interval.next(); // Wait for next tick.
            }
        }
    }

    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let addr = "0.0.0.0:4444";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    info!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream
            .peer_addr()
            .expect("connected streams should have a peer address");
        info!("Peer address: {}", peer);

        tokio::spawn(accept_connection(peer, stream));
    }
}

// async fn run(sender: &mut dyn SinkExt<Message>){
//     println!("In Thread");
//     println!("Windowsize: {}", WINDOW_BYTES);
//
//     let pid = utils::get_pid().unwrap();
//
//     println!("Pid: {}", pid);
//
//     let fb0_addr = utils::get_fb0addr(pid).unwrap();
//
//     println!("FB0: 0x{:X}", fb0_addr);
//     println!("FB0: {}", fb0_addr);
//
//     let reader = MemReader::new(pid as u32).unwrap();
//
//     let mut buf_a = [0u8; WINDOW_BYTES];
//     let mut buf_b = [0u8; WINDOW_BYTES];
//
//     let mut use_buffer_a = true;
//
//     loop {
//         let start_begin = SystemTime::now();
//
//         if use_buffer_a {
//             utils::fill_buffer(fb0_addr, &reader, &mut buf_a)
//         } else {
//             utils::fill_buffer(fb0_addr, &reader, &mut buf_b)
//         }.unwrap();
//         let read_time = start_begin.elapsed().unwrap().as_millis();
//
//         let start = SystemTime::now();
//         let equal = utils::check_equality(&buf_a, &buf_b);
//         let cmp_time = start.elapsed().unwrap().as_millis();
//         print!("Equality: {}  ", equal);
//         let start = SystemTime::now();
//         if !equal {
//             let encoded = if use_buffer_a {
//                 utils::encode(&buf_a, WIDTH)
//             } else {
//                 utils::encode(&buf_b, WIDTH)
//             };
//
//             sender.send(Message::Binary(encoded)).unwrap();
//         }
//
//         let enc_time = start.elapsed().unwrap().as_millis();
//
//         // println!("Read: {}", check_equality(&buf_a, &buf_b));
//
//         use_buffer_a = !use_buffer_a;
//         println!("Read: {:>3} ms, Cmp: {:>3} ms, Enc: {:>3} ms, All: {:>3} ms", read_time, cmp_time, enc_time, start_begin.elapsed().unwrap().as_millis());
//     }
// }

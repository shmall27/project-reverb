#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!("../bindings.rs");

use std::net::{UdpSocket};
use std::sync::{mpsc};
use std::thread;

unsafe fn decode_buffer_arr(message: [u8; 600]) -> () {
    let videoCodec: * const AVCodec = avcodec_find_decoder(AVCodecID_AV_CODEC_ID_HEVC);

    let codecContext: * mut AVCodecContext = avcodec_alloc_context3(videoCodec);

    avcodec_open2(codecContext, videoCodec, None);

    println!("{:?}", videoCodec);
    println!("{:?}", message)
}

fn main() -> std::io::Result<()> {
    {
        let socket = UdpSocket::bind("127.0.0.1:6000")?;
        let thread_socket = socket.try_clone().unwrap();
        let mut buf = [0; 600];
        let (tx, rx) = mpsc::channel::<[u8; 600]>();

        thread::spawn(move || {
            for message in rx {
                //send the message to a new udp connection
                thread_socket.send_to(&message, "127.0.0.1:4242").unwrap();
                unsafe { decode_buffer_arr(message); }
                
                // let decoded_buffer_arr = unsafe { decode_buffer_arr(&message) };
                // println!("{:?}", decoded_buffer_arr)
            }
        });

        loop {
            socket.recv_from(&mut buf)?;
            tx.send(buf).unwrap();
        }
    }
}
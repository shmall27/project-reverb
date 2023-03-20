#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!("../bindings.rs");

use std::net::{UdpSocket};
use std::error::Error;
use std::sync::{mpsc};
use std::thread;
use std::ptr;

const packetSize: u32 = 600 + AV_INPUT_BUFFER_PADDING_SIZE;

struct Decoder {
    buf: Box<[u8]>,
    packet: * mut AVPacket,
    frame: * mut AVFrame,
    codecContext: * mut AVCodecContext,
}

impl Decoder {
    unsafe fn new(packet_size: usize) -> Result<Self, Box<dyn Error>> {
        let video_codec = avcodec_find_decoder(AVCodecID_AV_CODEC_ID_HEVC);
        if video_codec.is_null() {
            return Err("HEVC codec not found".into());
        }

        let codec_context = avcodec_alloc_context3(video_codec);
        if codec_context.is_null() {
            return Err("Failed to allocate codec context".into());
        }

        let empty_dict_ptr: *mut *mut AVDictionary = ptr::null_mut();
        let ret = avcodec_open2(codec_context, video_codec,empty_dict_ptr);
        if ret < 0 {
            return Err(format!("Failed to open codec: {}", ret).into());
        }

        let frame = av_frame_alloc();
        if frame.is_null() {
            return Err("Failed to allocate frame".into());
        }

        let packet = av_packet_alloc();
        if packet.is_null() {
            return Err("Failed to allocate packet".into());
        }

        let buf_ptr = av_malloc(packet_size.try_into().unwrap()) as *mut u8;
        let buf = std::slice::from_raw_parts_mut(buf_ptr, packet_size).to_owned().into_boxed_slice();

        Ok(Self {
            buf,
            packet,
            frame,
            codecContext: codec_context,
        })
    }

    unsafe fn decode_buffer_arr(&mut self, buffer: &[u8]) -> Result<(), Box<dyn Error>> {
        self.buf[..buffer.len()].copy_from_slice(buffer);

        (*self.packet).data = self.buf.as_mut_ptr();
        (*self.packet).size = buffer.len().try_into().unwrap();

        let ret = avcodec_send_packet(self.codecContext, self.packet);
        if ret < 0 {
            return Err(format!("Failed to send packet: {}", ret).into());
        }

        let ret = avcodec_receive_frame(self.codecContext, self.frame);
        if ret < 0 {
            return Err(format!("Failed to receive frame: {}", ret).into());
        }

        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    {
        let socket = UdpSocket::bind("127.0.0.1:6000")?;
        let thread_socket = socket.try_clone().unwrap();
        let mut buf = [0; 600];
        let (tx, rx) = mpsc::channel::<[u8; 600]>();

        thread::spawn(move || {
            let mut decoder = unsafe { Decoder::new(packetSize as usize).unwrap() };
            for message in rx {
                unsafe {
                    decoder.decode_buffer_arr(&message);
                }
                //send the message to a new udp connection
                thread_socket.send_to(&message, "127.0.0.1:4242").unwrap();
            }
        });

        loop {
            socket.recv_from(&mut buf)?;
            tx.send(buf).unwrap();
        }
    }
}
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!("../bindings.rs");

extern crate sdl2;

mod utils;

use std::net::{UdpSocket};
use std::error::Error;
use std::sync::{mpsc, Arc};
use std::thread;
use std::ptr;
use std::time::Duration;

use sdl2::keyboard::Keycode;
use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;
// use sdl2::rect::Rect;

const packetSize: u32 = 600 + AV_INPUT_BUFFER_PADDING_SIZE;

struct FrameData {
    data: Vec<u8>,
    linesize: usize,
}

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

        // supposed to be allocated av_malloc, but it requires typecasting and is ugly
        let buf = vec![0; packet_size].into_boxed_slice();

        Ok(Self {
            buf,
            packet,
            frame,
            codecContext: codec_context,
        })
    }

    unsafe fn decode_buffer_arr(&mut self, buffer: &[u8]) -> Result<Vec<FrameData>, Box<dyn Error>>{
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

        let frame_height = (*self.frame).height as usize;

        let mut planes: Vec<FrameData> = Vec::new();

        for i in 0..8 {
            let data_ptr = (*self.frame).data[i];
            let linesize = (*self.frame).linesize[i] as usize;
            if data_ptr.is_null() || linesize == 0 {
                break;
            }

            let plane_height = if i == 0 { frame_height } else { frame_height / 2 };
            let plane_data = std::slice::from_raw_parts(data_ptr, linesize * plane_height);
            planes.push(FrameData {data: plane_data.to_vec(), linesize: linesize});
        }

        return Ok(planes);
    }

    unsafe fn width(&self) -> u32 {
        (*self.codecContext).width as u32
    }

    unsafe fn height(&self) -> u32 {
        (*self.codecContext).height as u32
    }
}

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:6000").unwrap();
    let thread_socket = socket.try_clone().unwrap();
    let mut buf = [0; 600];
    let (tx, rx) = mpsc::channel::<[u8; 600]>();


    thread::spawn(move || {
        loop {
            socket.recv_from(&mut buf).unwrap();
            tx.send(buf).unwrap();
        }
    });

    let (width_height_sender, width_height_receiver) = mpsc::channel::<(u32, u32)>();

    // create the thread-safe queue within the arc here
    let shared_queue = Arc::new(utils::thread_safe_queue::ThreadSafeQueue::<Vec<FrameData>>::new(300));
    // clone it and pass to the decoder thread
    let decoder_queue = shared_queue.clone();
    thread::spawn(move || {
        let mut decoder = unsafe { Decoder::new(packetSize as usize).unwrap() };

        let mut is_first = true;

        for message in rx {
            let frame_data = unsafe { decoder.decode_buffer_arr(&message) };
            match frame_data {
                Ok(frame_data) => {
                    if is_first {
                        let frame_width = unsafe { decoder.width() as u32 };
                        let frame_height = unsafe { decoder.height() as u32 };
                        width_height_sender.send((frame_width, frame_height)).unwrap();
                        is_first = false;
                    }
                    decoder_queue.push(frame_data);
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }

            //send the message to a new udp connection
            thread_socket.send_to(&message, "127.0.0.1:4242").unwrap();
        }
    });

    // clone it and pass to the sdl thread
    let display_queue = shared_queue.clone();
    
    let (frame_width, frame_height) = width_height_receiver.recv().unwrap();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", frame_width, frame_height)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    

    let mut canvas = window.into_canvas().build().unwrap();

    let texture_creator = canvas.texture_creator();

    // 800 x 600 is just a placeholder for now; need to use the frame.width and frame.height
    let mut video_texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::YV12, frame_width, frame_height)
        .unwrap();
    
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }

        let planes = display_queue.pop();

        for (i, plane) in planes.iter().enumerate() {
            let plane_slice = &plane.data[..];
            let plane_linesize = plane.linesize;
            
            let rect = match i {
                0 => sdl2::rect::Rect::new(0, 0, frame_width, frame_height),
                _ => sdl2::rect::Rect::new(0, 0, frame_width / 2, frame_height / 2),
            };
            
            video_texture
                .update(rect, plane_slice, plane_linesize)
                .unwrap();
        }

        canvas.copy(&video_texture, None, None).unwrap();
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
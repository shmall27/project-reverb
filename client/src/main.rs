#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!("../bindings.rs");

use std::ffi::{c_int, CString};
use std::net::{UdpSocket};
use std::sync::{mpsc};
use std::thread;
use std::ptr;

use cty::{uint8_t, c_void};

const packetSize: u32 = 600 + AV_INPUT_BUFFER_PADDING_SIZE;

unsafe fn get_decode_settings() -> () {
    let videoCodec: * const AVCodec = avcodec_find_decoder(AVCodecID_AV_CODEC_ID_HEVC);

    let codecContext: * mut AVCodecContext = avcodec_alloc_context3(videoCodec);

    let mut emptyDictionary: * mut AVDictionary = ptr::null_mut();
    let emptyDictPointer: * mut * mut AVDictionary = &mut emptyDictionary;

    avcodec_open2(codecContext, videoCodec, emptyDictPointer);

    let frame: * mut AVFrame = av_frame_alloc();

    let packet: * mut AVPacket = av_packet_alloc();

    let bufPtr: *mut c_void = av_malloc(packetSize.into());
}

unsafe fn decode_buffer_arr(bufferArr: [u8; 600]) -> () {

    let videoCodec: * const AVCodec = avcodec_find_decoder(AVCodecID_AV_CODEC_ID_HEVC);

    let codecContext: * mut AVCodecContext = avcodec_alloc_context3(videoCodec);

    let mut emptyDictionary: * mut AVDictionary = ptr::null_mut();
    let emptyDictPointer: * mut * mut AVDictionary = &mut emptyDictionary;

    avcodec_open2(codecContext, videoCodec, emptyDictPointer);

    let frame: * mut AVFrame = av_frame_alloc();

    let packet: * mut AVPacket = av_packet_alloc();

    let bufPtr: *mut c_void = av_malloc(packetSize.into());

    let buf: &mut [u8] = std::slice::from_raw_parts_mut(bufPtr as * mut uint8_t, packetSize.try_into().unwrap());

    for i in 1..600 {
        buf[i] = bufferArr[i]
    }

    let filename: * const i8 = CString::new("test").unwrap().as_ptr();
    let mimeType: * const i8 = CString::new("video/mp4").unwrap().as_ptr();


    let probeData: AVProbeData = AVProbeData {filename: filename, buf: bufPtr as * mut uint8_t, buf_size: packetSize.try_into().unwrap(), mime_type: mimeType};

    let format: * const AVInputFormat = av_probe_input_format(&probeData as * const _, 0);

    let streamStatus: c_int = av_read_frame(format, packet);

    println!("{:?}", packet);

    // let _packetFromData: c_int = av_packet_from_data(packet, bufPtr as * mut uint8_t, packetSize.try_into().unwrap());

    let _sendPacketResponse: c_int = avcodec_send_packet(codecContext, packet);

    let _recFrameResponse: c_int = avcodec_receive_frame(codecContext, frame);


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
            }
        });

        loop {
            socket.recv_from(&mut buf)?;
            tx.send(buf).unwrap();
            unsafe { decode_buffer_arr(buf); }
        }
    }
}
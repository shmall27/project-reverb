use std::net::{UdpSocket};
use std::sync::{mpsc};
use std::thread;

include!("../bindings.rs");

extern {
    fn decode_buffer_arr(buffer_arr: *const[u8; 600]);
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
                let decoded_buffer_arr = unsafe { decode_buffer_arr(&message) };
                println!("{:?}", decoded_buffer_arr)
            }
        });

        loop {
            socket.recv_from(&mut buf)?;
            tx.send(buf).unwrap();
        }
    }
}

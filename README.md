# project-reverb
A P2P dynamic live-streaming client (in-progress)

### What is the idea?
I mostly just want to learn Rust (and some C), so for fun, I wanted to create a way to stream a live-stream in a P2P fashion. The idea is that the host of the live-stream sends it to a few members of the audience. Those members then send the stream on to more members and so on. I think this is commonly referred to as "re-streaming." I quickly got stuck on decoding and displaying the video, so I haven't gotten to any of the networking stuff yet.

### Notes
- I put an example packet in the C code for a more simple demo with less setup
- - You shouldn't even have to run the Rust code or OBS to replicate the current error, but I included instructions below if you're interested in that part of the code.

### Setup Guide

**OBS Setup**
- Go to:
- - Settings -> Output -> Recording
- - Set the type to "Custom Output (FFmpeg)
- - FFmpeg Output Type = "Output to URL"
- - File path or URL = "udp://127.0.0.1:6000" (this just has to match the port in the Rust code)
- - I just kinda yolo'd the rest of the settings

**Install Dependencies:**
- brew install ffmpeg
- installing rust: https://doc.rust-lang.org/book/ch01-01-installation.html

**Compile C Code:**
- ```gcc -o video_player -L/usr/local/lib -lavcodec main.c```

**Compile Rust Code:**
- ```cargo run``` (i think that's it)
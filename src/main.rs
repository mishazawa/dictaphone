extern crate cpal;
extern crate hound;

use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use std::sync::mpsc;
use std::thread;
use std::path::Path;
use std::i16;
use std::u8;
use std::time::{SystemTime};
use std::io::{self, Write};
use std::env;


const WAV_AMP:f32 = i16::MAX as f32;
const STD_AMP:f32 = u8::MAX as f32 / 2.0;


fn main() {
  let mut write_to_stdout = true;
  let mut write_to_wav = true;
  for argument in env::args() {
    if argument == "--no-std" {
      write_to_stdout = false;
    }
    if argument == "--no-wav" {
      write_to_wav = false;
    }
    println!("{:?}", argument);
  }

  let host = cpal::default_host();
  let device = host.default_input_device().expect("No input device");
  let format = device.default_input_format().expect("No input format");
  let event_loop = host.event_loop();
  let stream_id = event_loop.build_input_stream(&device, &format).expect("No stream");
  event_loop.play_stream(stream_id).expect("No play");

  let audio_spec = hound::WavSpec {
    channels: format.channels,
    sample_rate: 44100,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int,
  };

  let mut name = "Voice_".to_string();
  if let Ok(time) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
    name.push_str(&time.as_millis().to_string());
  }
  name.push_str(".wav");

  let file_path: &Path = name.as_ref();

  let mut writer = match file_path.is_file() {
    true => hound::WavWriter::append(file_path).unwrap(),
    false => hound::WavWriter::create(file_path, audio_spec).unwrap(),
  };

  let (sound_tx, sound_rx) = mpsc::channel();
  let stdout = io::stdout();
  let mut handle = stdout.lock();
  thread::spawn(move || {
    event_loop.run(move |id, event| {
      let data = match event {
        Ok(data) => data,
        Err(err) => {
          eprintln!("an error occurred on stream {:?}: {}", id, err);
          return;
        }
      };
      match data {
        cpal::StreamData::Input { buffer: cpal::UnknownTypeInputBuffer::F32(buffer) } => {
          sound_tx.send(buffer.to_vec()).unwrap();
        },
        _ => {}
      }
    });
  });

  loop {
    if let Ok(data) = sound_rx.try_recv() {
      if write_to_stdout {
        let buff: Vec<u8> = data.clone().iter().map(|i| {
          ((i + 1.0) * STD_AMP) as u8
        }).collect();
        handle.write_all(&buff[..]).expect("Can't write to stdout");
      }
      if write_to_wav {
        data.iter().for_each(|sample| writer.write_sample((sample * WAV_AMP) as i16).unwrap());
      }
    }
  }
}

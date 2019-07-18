extern crate cpal;
extern crate hound;

use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use std::sync::mpsc;
use std::thread;
use std::path::Path;
use std::i16;
use std::time::{SystemTime};

const WAV_AMP:f32 = i16::MAX as f32;

fn main() {
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
      data.iter().for_each(|sample| writer.write_sample((sample * WAV_AMP) as i16).unwrap());
    }
  }
}

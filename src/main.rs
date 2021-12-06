mod audiodevices;

use audiodevices::{
    capture_output_audio, get_default_audio_output_device, get_output_audio_devices,
};
use cpal::{
    traits::{DeviceTrait, StreamTrait},
    Sample,
};

use flume::{unbounded, Receiver, Sender};

// Adapted from https://github.com/dheijl/swyh-rs/blob/44676eb543d4238311c87b68858589e60d0debe7/src/main.rs
fn main() {
    let mut audio_output_device =
        get_default_audio_output_device().expect("No default audio device");

    let audio_devices = get_output_audio_devices();
    let mut source_names: Vec<String> = Vec::new();
    for adev in audio_devices {
        let devname = adev.name().unwrap();
        if devname == "default" {
            audio_output_device = adev;
        }
        source_names.push(devname);
    }

    let (sender, receiver): (Sender<Vec<f32>>, Receiver<Vec<f32>>) = unbounded();

    let stream: cpal::Stream;
    match capture_output_audio(&audio_output_device, sender) {
        Some(s) => {
            stream = s;
            stream.play().unwrap();
        }
        None => {
            panic!("Unable to capture audio output");
        }
    }

    let mut nsamples = 0_i64;
    let mut sum_l = 0_i64;
    let mut sum_r = 0_i64;
    while let Ok(samples) = receiver.recv() {
        for (n, sample) in samples.iter().enumerate() {
            nsamples += 1;

            let i64sample = i64::from((*sample).to_i16());

            if n & 1 == 0 {
                sum_l += i64sample * i64sample;
            } else {
                sum_r += i64sample * i64sample;
            }

            //max is 16384
            let rms_l = (((sum_l / nsamples) as f64).sqrt()) * 255.0 / 16384.0;
            let rms_r = (((sum_r / nsamples) as f64).sqrt()) * 255.0 / 16384.0;

            println!("{} - {}", rms_l, rms_r);

            // Uncommenting this sleep makes all data become 0
            // thread::sleep(Duration::from_millis(1));

            nsamples = 0;
            sum_l = 0;
            sum_r = 0;
        }
    }
}

use std::{io::Read, thread::JoinHandle};
use symphonia::core::{
    audio::SampleBuffer, 
    codecs::DecoderOptions,
    formats::FormatOptions, 
    io::{MediaSourceStream, ReadOnlySource}, 
    meta::MetadataOptions, probe::Hint
};
use crate::{distributions::SpeedFactorDistribution, receiver::ReadableReceiver};

pub struct AudioChunk {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub speed_factor: f32
}


struct StreamingData {
    sample_buf: SampleBuffer<f32>,
    sample_rate: u32
}

fn download_thread(url: String, sender: flume::Sender<Vec<u8>>, buffer_size: usize) {
    let agent: ureq::Agent = ureq::Agent::config_builder().max_response_header_size(1_000_000).build().into();
    let mut res = agent.get(url).call().unwrap();
    let res_body = res.body_mut();
    let mut reader = res_body.with_config().reader();
    let mut buffer = vec![0u8; buffer_size];
    while let Ok(n) = reader.read(&mut buffer) {
        if n == 0 {
            break;
        }
        sender.send(buffer[..n].to_vec()).unwrap();
    }
}

fn process_thread(receiver: ReadableReceiver, sender: flume::Sender<AudioChunk>) {
    let target_sample_size = 512 * 256 - 1;
    let target_sample_rate = 16_000;
    let speed_factor_distribution = SpeedFactorDistribution { min: 0.75, max: 1.25 };

    let reader = std::io::BufReader::new(receiver);
    let mss = MediaSourceStream::new(Box::new(ReadOnlySource::new(reader)), Default::default());

    let mut hint = Hint::new();
    hint.with_extension("mp3");

    let format_opts: FormatOptions = Default::default();
    let metadata_opts: MetadataOptions = Default::default();
    let decoder_opts: DecoderOptions = Default::default();

    // Probe the media source stream for a format.
    let probed = symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts).unwrap();

    // Get the format reader yielded by the probe operation.
    let mut format = probed.format;

    // Get the default track.
    let track = format.default_track().unwrap();

    // Create a decoder for the track.
    let mut decoder =
        symphonia::default::get_codecs().make(&track.codec_params, &decoder_opts).unwrap();

    // Store the track identifier, we'll use it to filter packets.
    let track_id = track.id;

    let mut sample_count = 0;
    let mut streaming_data = None;

    let target_seconds = (target_sample_size as f32) / (target_sample_rate as f32);

    let mut current_speed_factor = speed_factor_distribution.next();

    let mut all_samples = vec![];

    loop {
        // Get the next packet from the format reader.
        let Ok(packet) = format.next_packet() else {
            break;
        };

        // If the packet does not belong to the selected track, skip it.
        if packet.track_id() != track_id {
            continue;
        }

        // Decode the packet into audio samples, ignoring any decode errors.
        match decoder.decode(&packet) {
            Ok(audio_buf) => {
                // If this is the *first* decoded packet, create a sample buffer matching the
                // decoded audio buffer format.
                if streaming_data.is_none() {
                    // Get the audio buffer specification.
                    let spec = *audio_buf.spec();

                    // Get the capacity of the decoded buffer. Note: This is capacity, not length!
                    let duration = audio_buf.capacity() as u64;

                    streaming_data = Some(StreamingData {
                        sample_rate: spec.rate,
                        sample_buf: SampleBuffer::<f32>::new(duration, spec)
                    });
                }

                if let Some(data) = &mut streaming_data {

                    // Copy the decoded audio buffer into the sample buffer in an interleaved format.
                    data.sample_buf.copy_interleaved_ref(audio_buf);

                    // The samples may now be access via the `samples()` function.
                    sample_count += data.sample_buf.samples().len();

                    // Add the samples to the running total
                    all_samples.extend_from_slice(data.sample_buf.samples());

                    // Idea: based on our current speed augmentation, make an estimate
                    // for the number of samples we need to collect at this higher sample rate.
                    // Then, once we collect that many samples, immediately process them and
                    // pass it on to the Sender

                    let required_seconds = target_seconds * current_speed_factor;
                    let required_samples = (required_seconds * (data.sample_rate as f32)) as usize;

                    if sample_count >= required_samples {

                        let cut_samples = &all_samples[..required_samples];
                        
                        sender.send(AudioChunk {
                            samples: cut_samples.to_vec(),
                            sample_rate: target_sample_rate,
                            speed_factor: current_speed_factor,
                        }).unwrap();

                        // grab a new speed factor
                        current_speed_factor = speed_factor_distribution.next();

                        // remove the first required_samples from the buffer
                        let remaining_samples = sample_count - required_samples;

                        if remaining_samples > 0 {
                            all_samples.copy_within(required_samples.., 0);
                            all_samples.truncate(remaining_samples);
                        } else {
                            all_samples.clear();
                        }

                        sample_count = remaining_samples;
                    }
                }
            }
            Err(symphonia::core::errors::Error::DecodeError(_)) => (),
            Err(_) => break,
        }
    }
}

fn worker_thread(url_receiver: flume::Receiver<String>, chunk_sender: flume::Sender<AudioChunk>) {
    while let Ok(url) = url_receiver.recv() {
        let url_clone = url.clone();

        let (stream_sender, stream_receiver) = flume::bounded(128);
        let stream_receiver = ReadableReceiver::new(stream_receiver, 8192);
        let download_thread = std::thread::spawn(move || download_thread(url, stream_sender, 8192));

        let chunk_sender = chunk_sender.clone();
        let process_thread = std::thread::spawn(move || process_thread(stream_receiver, chunk_sender));

        let _ = download_thread.join();
        println!("Finished downloading {}", &url_clone);

        let _ = process_thread.join();
        println!("Finished processing {}", &url_clone);
    }
}

pub struct AudioTrackDownloadPool {
    workers: Vec<JoinHandle<()>>,
    chunk_receiver: flume::Receiver<AudioChunk>,
}

impl AudioTrackDownloadPool {
    pub fn new(num_workers: usize, urls: Vec<String>) -> Self {
        let (url_sender, url_receiver) = flume::unbounded();

        for url in urls {
            url_sender.send(url).unwrap();
        }

        let (chunk_sender, chunk_receiver) = flume::bounded(128);

        let mut workers = vec![];

        for _ in 0..num_workers {
            let url_receiver = url_receiver.clone();
            let chunk_sender = chunk_sender.clone();
            let handle = std::thread::spawn(move || worker_thread(url_receiver, chunk_sender));
            workers.push(handle);
        }

        Self {
            workers,
            chunk_receiver
        }
    }

    pub fn next(&self) -> Option<AudioChunk> {
        self.chunk_receiver.recv().ok()
    }

    pub fn join(self) {
        for worker in self.workers {
            worker.join().unwrap();
        }
    }
}

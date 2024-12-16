fn main() {
    let download_pool = librivox_streamer::pool::AudioTrackDownloadPool::new(3, vec![
        "https://www.archive.org/download/count_monte_cristo_0711_librivox/count_of_monte_cristo_002_dumas.mp3".to_string(),
        "https://www.archive.org/download/count_monte_cristo_0711_librivox/count_of_monte_cristo_020_dumas.mp3".to_string(),
        "https://www.archive.org/download/count_monte_cristo_0711_librivox/count_of_monte_cristo_003_dumas.mp3".to_string()
    ]);

    while let Some(chunk) = download_pool.next() {
        println!("Received chunk with {} samples, step_factor = {}.", chunk.samples.len(), chunk.step_factor);
    }
}
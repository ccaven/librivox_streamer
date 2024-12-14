import numpy as np
from librivox_streamer import PyAudioTrackDownloadPool

def chunk_iter():
    urls = [
        "https://www.archive.org/download/count_monte_cristo_0711_librivox/count_of_monte_cristo_002_dumas.mp3",
        "https://www.archive.org/download/count_monte_cristo_0711_librivox/count_of_monte_cristo_020_dumas.mp3",
        "https://www.archive.org/download/count_monte_cristo_0711_librivox/count_of_monte_cristo_003_dumas.mp3"
    ]

    downloader = PyAudioTrackDownloadPool(3, urls)
    
    while (item := downloader.next()) is not None:
        yield item

for chunk in chunk_iter():
    samples = np.frombuffer(chunk.samples_bytes(), dtype=np.float32)
    print(f"Received chunk of shape {samples.shape}")

print("Finished downloading.")
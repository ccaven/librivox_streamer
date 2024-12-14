class PyAudioChunk:
    """
    Stores the data for a single 8.3 second chunk
    """
    
    def __len__(self) -> int:
        """
        Returns the length, in samples, of the audio chunk
        """
    
    def sample_rate(self) -> int:
        """
        Returns the sampling rate of the audio
        """
    
    def speed_factor(self) -> float:
        """
        Returns the desired speed factor of the samples
        """
    
    def samples_bytes(self) -> bytes:
        """
        Returns an immutable reference to the source float32 data
        """

class PyAudioTrackDownloadPool:
    """
    Manages multiple threads which download and chunk audio files in parallel.
    """
    
    def __init__(self, num_workers: int, urls: list[str]):
        """
        Starts the download and chunking process
        """
    
    def next(self) -> PyAudioChunk | None:
        """
        Returns the next available chunk. This method blocks the main thread. Returns None if all downloads are finished.
        """
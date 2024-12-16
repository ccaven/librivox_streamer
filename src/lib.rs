use pyo3::prelude::*;

mod receiver;
mod distributions;
pub mod pool;

#[pyclass]
pub struct PyAudioChunk {
    pub inner: pool::AudioChunk
}

#[pymethods]
impl PyAudioChunk {
    fn __len__(&self) -> usize {
        self.inner.samples.len()
    }

    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate
    }

    fn speed_factor(&self) -> f32 {
        self.inner.speed_factor
    }

    fn step_factor(&self) -> i32 {
        self.inner.step_factor
    }

    fn samples_bytes<'a>(&'a self) -> &'a [u8] {
        let samples = &self.inner.samples;
        bytemuck::cast_slice(samples)
    }
}

#[pyclass]
pub struct PyAudioTrackDownloadPool {
    inner: pool::AudioTrackDownloadPool
}

#[pymethods]
impl PyAudioTrackDownloadPool {
    #[new]
    pub fn new(num_workers: usize, urls: Vec<String>) -> Self {
        Self {
            inner: pool::AudioTrackDownloadPool::new(num_workers, urls)
        }
    }

    pub fn next(&self) -> Option<PyAudioChunk> {
        self.inner.next().map(|inner| PyAudioChunk { inner })
    }
}

#[pymodule]
fn librivox_streamer(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyAudioChunk>()?;
    m.add_class::<PyAudioTrackDownloadPool>()?;
    Ok(())
}

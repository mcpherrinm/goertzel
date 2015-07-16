// This (implicitly/naively) uses a rectangular window.  Some way to set up a window function
// will be needed probably -- if mutating your samples before calling this isn't sufficient.
use std::f32;
use std::f32::consts::PI;

/// Set up parameters (and some precomputed values for those).
#[derive(Clone, Copy)]
pub struct Parameters {
    // Parameters we're working with
    window_size: u32,
    // Precomputed value:
    sine: f32,
    cosine: f32,
    term_coefficient: f32,
}

pub struct Partial {
    params: Parameters,
    //count: u32,
    prev: f32,
    prevprev: f32,
}

impl Parameters {
    pub fn new(target_freq: f32, sample_rate: f32, window_size: u32) -> Self {
        let k = target_freq * (window_size as f32) / sample_rate;
        let omega = (f32::consts::PI * 2. * k) / (window_size as f32);
        let cosine = omega.cos();
        Parameters {
            window_size: window_size,
            sine: omega.sin(),
            cosine: cosine,
            term_coefficient: 2. * cosine,
        }
    }

    pub fn start(self) -> Partial {
        Partial{ params: self, prev: 0., prevprev: 0. }
    }
}

impl Partial {
    pub fn add(mut self, samples: &[i16]) -> Self {
        for &sample in samples {
            let this = self.params.term_coefficient * self.prev - self.prevprev + (sample as f32);
            self.prevprev = self.prev;
            self.prev = this;
        }
        self
    }
    pub fn finish(self) -> (f32, f32) {
        let real = self.prev - self.prevprev * self.params.cosine;
        let imag = self.prevprev * self.params.sine;
        (real, imag)
    }

    pub fn finish_mag(self) -> f32 {
        let (real, imag) = self.finish();
        (real*real + imag*imag).sqrt()
    }
}

#[test]
fn zero_data() {
    let p = Parameters::new(1800., 8000., 256);
    assert!(p.start().add(&[0; 256]).finish_mag() == 0.);
    assert!(p.start().add(&[0; 128]).add(&[0;128]).finish_mag() == 0.);
}

#[test]
fn one_sine() {
    let mut buf = [0; 8000];
    // Generate a 1 second sine wave at 1800 hz
    // Using 8khz sample rate: Generate 8k samples,
    // map them into our second (zero to one):
    let step = 1. / 8000.;
    for sample in (0 .. 8000) {
        let time = sample as f32 * step;
        buf[sample] = ((time * 1800. * 2. * PI).sin()*std::i16::MAX as f32) as i16;
    }

    let p = Parameters::new(1800., 8000., 256);
    let mag1800 = p.start().add(&buf[0..256]).finish_mag();
    println!("1800: {}", mag1800);
    for freq in (0 .. 30).map(|x| (x * 100) as f32) {
        let p = Parameters::new(freq, 8000., 256);
        let mag = p.start().add(&buf[0..256]).finish_mag();
        println!("{}: {}", freq, mag);
        if freq != 1800. {
            assert!(mag1800 > 10.*mag);
        }
    }

}


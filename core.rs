use audio_open::MySample;
use imageproc::image;
use rodio::Sample;
pub use visual_signal::ViewSignal;

mod visual_signal {
    use std::fmt::{Debug, Display};
    use std::ops::{AddAssign, Div, Mul};

    use cpal::{FromSample, Sample, SampleFormat, SizedSample};
    use imageproc::drawing::{draw_antialiased_line_segment_mut, Canvas};
    use imageproc::image::{DynamicImage, ImageBuffer, Pixel, Rgb};
    use imageproc::pixelops::interpolate;

    use self::audio_process::draw_wave;

    use super::*;

    pub struct ViewSignal {
        image: ImageBuffer<Rgb<u8>, Vec<u8>>,
    }

    impl ViewSignal {
        pub fn new<T: Sample + Default + SizedSample + FromSample<T> + Debug + AddAssign>(
            sound: &[T],
            desired_size: [usize; 2],
            wave_color: [u8; 3],
            background_color: [u8; 3],
        ) -> Self
        where
            f32: From<T>,
        {
            let height = desired_size[1] as f32;
            let width = desired_size[0];

            let mut buffer = vec![255; desired_size[0] * height as usize * 3];
            let channel_1 = vec![background_color[0]; width * height as usize];
            let channel_2 = vec![background_color[1]; width * height as usize];
            let channel_3 = vec![background_color[2]; width * height as usize];

            buffer.chunks_mut(3).enumerate().for_each(|(i, dst)| {
                dst[0] = channel_1[i];
                dst[1] = channel_2[i];
                dst[2] = channel_3[i];
            });

            let mut dst_image = ImageBuffer::from_raw(width as u32, height as u32, buffer).unwrap();

            let color = Rgb(wave_color);
            let highest: f32 = audio_process::wave_height_ratio::<T, f32>(sound);
            let wave_ratio = 1.0 / highest;

            draw_wave(sound, wave_ratio, desired_size, &mut dst_image, wave_color);

            Self { image: dst_image }
        }
        pub fn save(&self, file_name: &str) {
            self.image.save(file_name).unwrap();
        }

        pub fn convert<T>(&self, convert: impl FnOnce(&[u8], [usize; 2]) -> T) -> T {
            convert(
                self.image.as_raw(),
                [self.image.width() as usize, self.image.height() as usize],
            )
        }
        pub fn to_bytes(&self) -> Vec<u8> {
            self.image.to_vec()
        }
        pub fn as_bytes(&self) -> &[u8] {
            self.image.as_raw()
        }
    }
}

mod audio_process {
    use imageproc::{image::Rgb, pixelops::interpolate};
    use std::{
        fmt::{Debug, Display},
        ops::{AddAssign, Div, Mul},
    };

    use cpal::{FromSample, Sample, SizedSample};
    use imageproc::image::ImageBuffer;

    use super::*;
    use imageproc::drawing::draw_antialiased_line_segment_mut;

    pub fn find_highest_sample<T: FromSample<T> + SizedSample + Sample + AddAssign + Default>(
        samples: &[T],
    ) -> T {
        let mut highest_value = T::default();
        for sample in samples {
            let s: T = T::from_sample(*sample);
            if s > highest_value {
                highest_value += s;
            }
        }

        highest_value
    }

    pub fn wave_height_ratio<
        T: Sample + Default + SizedSample + FromSample<T> + Debug + AddAssign + Into<U>,
        U,
    >(
        sound: &[T],
    ) -> U {
        let highest = audio_process::find_highest_sample::<T>(sound);

        highest.into()
    }
    pub fn draw_wave<T: Copy>(
        sound: &[T],
        wave_ratio: f32,
        desired_size: [usize; 2],
        image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
        wave_color: [u8; 3],
    ) where
        f32: From<T>,
    {
        let sample_len = sound.len();
        let height = desired_size[1] as f32;
        let wave_color = Rgb(wave_color);
        for (i, s) in sound.iter().enumerate() {
            let s: f32 = T::into(*s);
            let x_pos_ratio = i as f32 / sample_len as f32;

            let im_width: i32 = (x_pos_ratio * desired_size[0] as f32) as i32;

            let s = s * wave_ratio;
            let start = (im_width, height as i32 / 2);
            if i % 2 == 0 {
                let end = (im_width, height as i32 / 2 + (height / 2.0 * s) as i32);
                draw_antialiased_line_segment_mut(image, start, end, wave_color, interpolate);
            } else {
                let end = (
                    im_width,
                    height as i32 / 2 - (height as f32 / 2.0 * s) as i32,
                );
                draw_antialiased_line_segment_mut(image, start, end, wave_color, interpolate);
            }
        }
    }
}

mod audio_open {

    use std::fs::File;
    use std::io::BufReader;
    use std::time::Duration;

    use rodio::{source::Source, Decoder};

    pub struct MySample {
        pub samples: Vec<f32>,
        pub duration: Duration,
    }

    impl MySample {
        pub fn new(file_path: &str) -> Self {
            let file = BufReader::new(File::open(file_path).unwrap());
            let source = Decoder::new(file).unwrap();

            let sample_rate = source.sample_rate();
            let channels = source.channels();

            let mut samples: Vec<f32> = vec![];

            for (i, s) in source.convert_samples::<f32>().step_by(1).enumerate() {
                samples.push(s)
            }

            let duration = (samples.len() / sample_rate as usize) / channels as usize;
            let duration_secs = std::time::Duration::from_secs(duration as u64);
            MySample {
                samples,
                duration: duration_secs,
            }
        }
        pub fn convert_duration_to_width(&self) -> usize {
            self.samples.len() / 100
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn main() {
        let sample = MySample::new(
            "/home/camille/Documents/rust/sound-wave-image/ressources/pencil_lines-91555.mp3",
        );

        let view = ViewSignal::new(
            &sample.samples,
            [8000 * 2, 4000 * 2],
            [255, 0, 0],
            [213, 10, 255],
        );
        view.save("/home/camille/Documents/rust/sound-wave-image/ressources/test_22.png");
    }
}

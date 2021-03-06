#![allow(unknown_lints)]
#![deny(warnings)]
extern crate image;

mod convolution;

pub use convolution::{Convolution, ConvolutionError};

use image::{GrayImage, ImageBuffer};

pub enum EdgeMode {
    Extend,
    Wrap,
    Mirror,
}

pub fn convolve(
    image_in: &GrayImage,
    convolution: &Convolution,
    edge_mode: &EdgeMode,
) -> Result<GrayImage, ConvolutionError> {
    let mut build = Vec::with_capacity((image_in.width() * image_in.height()) as usize);

    for (x, y, _) in image_in.enumerate_pixels() {
        let mut this_pixel = 0.0;
        let offset = (convolution.get_size() as i64) / 2;
        for y_offset in -offset..offset + 1 {
            for x_offset in -offset..offset + 1 {
                let p = get_pixel(
                    image_in,
                    i64::from(x) + x_offset,
                    i64::from(y) + y_offset,
                    edge_mode,
                );
                this_pixel += f64::from(p) *
                    convolution[((x_offset + offset) as usize, (y_offset + offset) as usize)];
            }

        }

        build.push(convolution.compute_adjusted_pixel_value(this_pixel));
    }

    Ok(
        ImageBuffer::from_raw(image_in.width(), image_in.height(), build).unwrap(),
    )
}

fn get_pixel(image: &image::GrayImage, x: i64, y: i64, edge_mode: &EdgeMode) -> u8 {
    let (proj_x, proj_y) = edge_project(x, y, image.width(), image.height(), edge_mode);

    image.get_pixel(proj_x, proj_y).data[0]
}

fn edge_project(x: i64, y: i64, width: u32, height: u32, edge_mode: &EdgeMode) -> (u32, u32) {
    let width = i64::from(width);
    let height = i64::from(height);

    if x >= 0 && x <= width - 1 && y >= 0 && y <= height - 1 {
        return (x as u32, y as u32);
    }

    match *edge_mode {
        EdgeMode::Extend => {
            let ret_x = if x < 0 {
                0
            } else if x > width - 1 {
                width - 1
            } else {
                x
            };


            let ret_y = if y < 0 {
                0
            } else if y > height - 1 {
                height - 1
            } else {
                y
            };

            (ret_x as u32, ret_y as u32)
        }
        EdgeMode::Wrap => {
            let ret_x = if x < 0 {
                x + width
            } else if x > width - 1 {
                x - width
            } else {
                x
            };


            let ret_y = if y < 0 {
                y + height
            } else if y > height - 1 {
                y - height
            } else {
                y
            };

            (ret_x as u32, ret_y as u32)
        }
        EdgeMode::Mirror => {
            let ret_x = if x < 0 {
                -x - 1
            } else if x > width - 1 {
                -x + (2 * width) - 1
            } else {
                x
            };


            let ret_y = if y < 0 {
                -y - 1
            } else if y > height - 1 {
                -y + (2 * height) - 1
            } else {
                y
            };

            (ret_x as u32, ret_y as u32)
        }
    }
}

#[cfg(test)]
mod tests {

    #[derive(PartialEq, Eq, Debug)]
    struct TestableGrayImg {
        pixels: Vec<u8>,
    }

    use super::{convolve, EdgeMode, Convolution};
    use image::{self, GrayImage};

    #[test]
    fn gaussian_blur_blurs() {
        let base_image = image::open("img/sam.jpg").expect("sam.jpg failed to open");

        let gaussian_convoloution = Convolution::new(
            &[
                0.003765,
                0.015019,
                0.023792,
                0.015019,
                0.003765,
                0.015019,
                0.059912,
                0.094907,
                0.059912,
                0.015019,
                0.023792,
                0.094907,
                0.150342,
                0.094907,
                0.023792,
                0.015019,
                0.059912,
                0.094907,
                0.059912,
                0.015019,
                0.003765,
                0.015019,
                0.023792,
                0.015019,
                0.003765,
            ],
        ).expect("making a convolution");

        let actual = convolve(&base_image.to_luma(), &gaussian_convoloution, &EdgeMode::Extend)
            .expect("unwrapping image");
        let expected = image::open("img/gaussian.png")
            .expect("loading guassian.png")
            .to_luma();

        assert_eq!(testable_repr(actual), testable_repr(expected));
    }

    #[test]
    fn identity_convolution_returns_same_image() {
        let base_image = image::open("img/sam.jpg").expect("sam.jpg failed to open");
        let identity_convoloution = Convolution::new(&[1.0]).expect("making a convolution");

        let gray_base = base_image.clone().to_luma();

        assert_eq!(
            testable_repr(
                convolve(&base_image.to_luma(), &identity_convoloution, &EdgeMode::Extend)
                    .expect("unwrapping image"),
            ),
            testable_repr(gray_base)
        );
    }

    fn testable_repr(img: GrayImage) -> TestableGrayImg {
        let mut build = Vec::with_capacity((img.width() * img.height()) as usize);
        for pixel in img.pixels() {
            build.push(pixel.data[0]);
        }

        TestableGrayImg { pixels: build }
    }
}

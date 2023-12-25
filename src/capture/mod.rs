#[cfg(target_os = "macos")]
use std::os::macos::raw;

use log::info;
use screenshots::image::{
    self, buffer::ConvertBuffer, imageops::resize, imageops::FilterType::Triangle, RgbImage, RgbaImage,
};

use crate::common::color::Color;
use crate::common::PixelRect;

/// retures Ok(buf) on success
/// buf contains pixels in [b:u8, g:u8, r:u8, a:u8] format, as an `[[i32;width];height]`.
pub fn capture_absolute(
    PixelRect {
        left,
        top,
        width,
        height,
    }: &PixelRect,
) -> Result<RgbImage, String> {
    let screen = screenshots::Screen::all().expect("cannot get DisplayInfo")[0];
    let image = screen
        .capture_area(*left, *top, *width as u32, *height as u32)
        .unwrap();
    Ok(image::DynamicImage::ImageRgba8(image).into_rgb8())
}

pub fn capture_absolute_image(
    PixelRect {
        left,
        top,
        width,
        height,
    }: &PixelRect,
) -> Result<image::RgbImage, String> {
    // simply use the first screen.
    // todo: multi-screen support
    let screen = screenshots::Screen::all().expect("cannot get DisplayInfo")[0];
    let image = screen
        .capture_area(*left, *top, *width as u32, *height as u32)
        .expect("capture failed");

    Ok(image::DynamicImage::ImageRgba8(image).into_rgb8())
}

pub fn get_color(x: u32, y: u32) -> Color {
    let im = capture_absolute(&PixelRect {
        left: x as i32,
        top: y as i32,
        width: 1,
        height: 1,
    })
    .unwrap();
    let pixel = im.get_pixel(0, 0);
    Color::from(pixel[0], pixel[1], pixel[2])
}

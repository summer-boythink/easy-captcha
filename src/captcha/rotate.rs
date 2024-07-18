use super::CaptchaOption;
use base64::{prelude::BASE64_STANDARD, Engine};
use image::{
    imageops::FilterType, open, DynamicImage, GenericImageView, ImageBuffer, ImageFormat, Pixel,
    Rgb, Rgba,
};
use imageproc::geometric_transformations::{rotate_about_center, Interpolation};
use rand::{distributions::Alphanumeric, seq::SliceRandom, Rng};
use std::{
    collections::HashMap,
    io::Cursor,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct RotateAppState {
    /// 旋转验证码基础配置
    option: CaptchaOption,
    /// 用于检测对应的key所需要的旋转的角度
    pub check_map: Arc<Mutex<HashMap<String, (f32, f32)>>>,
    /// 图片格式
    image_format: ImageFormat,
}

impl RotateAppState {
    pub fn new(option: CaptchaOption, image_format: ImageFormat) -> Self {
        RotateAppState {
            option,
            check_map: Arc::new(Mutex::new(HashMap::new())),
            image_format,
        }
    }

    pub fn generate_captcha(&self) -> Option<String> {
        let all_images_url = &self.option.images.clone();
        let leeway = self.option.leeway;
        let mut rng = rand::thread_rng();
        let image_path = all_images_url.choose(&mut rng).expect("No images provided");

        let img = open(image_path).expect("Failed to open image");

        // 计算圆形区域的直径和中心点
        let diameter = std::cmp::min(img.width(), img.height());
        let radius = diameter / 2;
        let center_x = img.width() / 2;
        let center_y = img.height() / 2;

        // 创建一个全透明的图像
        let mut circle_img = ImageBuffer::from_pixel(diameter, diameter, Rgba([0, 0, 0, 0]));

        // 将圆形区域从原图中复制到新图像
        for y in 0..diameter {
            for x in 0..diameter {
                let dx = x as i32 - radius as i32;
                let dy = y as i32 - radius as i32;
                if dx * dx + dy * dy <= (radius as i32 * radius as i32) {
                    let pixel = img.get_pixel(center_x - radius + x, center_y - radius + y);
                    circle_img.put_pixel(x, y, pixel);
                }
            }
        }

        let random_angle: f32 = rng.gen_range(-180.0..180.0);
        let rotated = rotate_about_center(
            &DynamicImage::ImageRgba8(circle_img).to_rgba8(),
            random_angle.to_radians(),
            Interpolation::Bilinear,
            Rgba([0, 0, 0, 0]),
        );

        // 移除旋转后的图像周围的透明边缘
        let mut final_img = ImageBuffer::from_pixel(diameter, diameter, Rgba([0, 0, 0, 0]));
        for y in 0..diameter {
            for x in 0..diameter {
                let pixel = rotated.get_pixel(x, y);
                if pixel[3] != 0 {
                    final_img.put_pixel(x, y, pixel.to_rgba());
                }
            }
        }
        // 调整大小
        let resized_img = DynamicImage::ImageRgba8(final_img).resize_exact(
            self.option.width.clone().try_into().unwrap(),
            self.option.height.clone().try_into().unwrap(),
            FilterType::Lanczos3,
        );

        let mut buffer = Cursor::new(Vec::new());
        resized_img
            .write_to(&mut buffer, self.image_format)
            .expect("Failed to write image");

        let hash_key: String = (0..32).map(|_| rng.sample(Alphanumeric) as char).collect();
        self.check_map.lock().unwrap().insert(
            hash_key.clone(),
            (random_angle - leeway, random_angle + leeway),
        );
        Some("data:image/png;base64,".to_owned() + &BASE64_STANDARD.encode(&buffer.into_inner()))
    }

    pub fn check_captcha(&self, hash_key: String, rotate: f32) -> bool {
        if let Some(&(min_angle, max_angle)) = self.check_map.lock().unwrap().get(&hash_key) {
            rotate >= min_angle && rotate <= max_angle
        } else {
            false
        }
    }
}

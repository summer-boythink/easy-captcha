pub mod rotate;

use clap::{Parser, ValueEnum};

#[derive(Parser, Debug, Clone)]
#[command(
    name = "easy-captcha",
    version = "1.0",
    about = "Generates a rotated captcha and outputs as Base64"
)]
pub struct CaptchaOption {
    #[arg(short, long, value_name = "IMAGES", required = true)]
    images: Vec<String>,

    #[arg(short, long, value_name = "WIDTH", required = true)]
    width: i32,

    #[arg(short, long, value_name = "HEIGHT", required = true)]
    height: i32,

    #[arg(short, long, value_name = "LEEWAY", required = true)]
    leeway: f32,

    #[arg(long, value_name = "TYPE", required = true)]
    captcha_type: CaptchaType,
}

#[derive(Debug)]
struct awe {
    
}

#[derive(Debug, Clone, ValueEnum)]
pub enum CaptchaType {
    Rotate,
    Slide,
}

impl CaptchaOption {
    pub fn new(
        images: Vec<String>,
        width: i32,
        height: i32,
        leeway: f32,
        captcha_type: CaptchaType,
    ) -> Self {
        CaptchaOption {
            images,
            width,
            height,
            leeway,
            captcha_type,
        }
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub timestamp_ms: u64,
}

impl CaptureFrame {
    pub fn save_as_png(&self, path: &str) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let rgba = self.to_rgba();
        let img = image::RgbaImage::from_raw(self.width, self.height, rgba)
            .ok_or_else(|| "Failed to create image")?;
        img.save(path)?;
        Ok(())
    }

    fn to_rgba(&self) -> Vec<u8> {
        // BGRA to RGBA conversion
        let mut rgba = Vec::with_capacity(self.data.len());
        for chunk in self.data.chunks(4) {
            if chunk.len() == 4 {
                rgba.push(chunk[2]); // R
                rgba.push(chunk[1]); // G
                rgba.push(chunk[0]); // B
                rgba.push(chunk[3]); // A
            }
        }
        rgba
    }
}

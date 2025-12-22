use windows::{
    Graphics::Imaging::*,
    Storage::Streams::*,
};
use image::{DynamicImage, GenericImageView, imageops::FilterType};
use std::io::Cursor;

pub struct OcrEngine {
    engine: Option<windows::Media::Ocr::OcrEngine>,
}

impl OcrEngine {
    pub fn new() -> Self {
        let engine = windows::Media::Ocr::OcrEngine::TryCreateFromUserProfileLanguages().ok();
        Self { engine }
    }

    pub fn process_image(&self, img: image::RgbaImage, debug: bool) -> String {
        let engine = match &self.engine {
            Some(e) => e,
            None => return String::new(),
        };

        // 1. Preprocess: Convert to grayscale and Upscale
        let mut img = DynamicImage::ImageRgba8(img);
        
        // Convert to grayscale to reduce noise
        let gray = img.grayscale();
        
        // Upscale (2x) using Lanczos3 filter for high quality
        let (w, h) = gray.dimensions();
        let upscaled = gray.resize(w * 2, h * 2, FilterType::Lanczos3);

        if debug {
            let _ = upscaled.save("debug_ocr.png");
        }

        // 2. Convert to SoftwareBitmap via Stream (BMP format)
        let mut buf = Vec::new();
        let mut cursor = Cursor::new(&mut buf);
        upscaled.write_to(&mut cursor, image::ImageOutputFormat::Bmp).ok();
        
        let stream = InMemoryRandomAccessStream::new().unwrap();
        let writer = DataWriter::CreateDataWriter(&stream).unwrap();
        writer.WriteBytes(&buf).unwrap();
        writer.StoreAsync().unwrap().get().unwrap();
        writer.FlushAsync().unwrap().get().unwrap();
        
        let decoder = BitmapDecoder::CreateAsync(&stream).unwrap().get().unwrap();
        let software_bitmap = decoder.GetSoftwareBitmapAsync().unwrap().get().unwrap();

        // 2. Recognize
        let result = engine.RecognizeAsync(&software_bitmap).unwrap().get().unwrap();
        
        // 3. Extract Text
        result.Text().unwrap().to_string()
    }
}

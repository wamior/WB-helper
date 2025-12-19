use windows::{
    Graphics::Imaging::*,
    Storage::Streams::*,
};
use image::DynamicImage;
use std::io::Cursor;

pub struct OcrEngine {
    engine: Option<windows::Media::Ocr::OcrEngine>,
}

impl OcrEngine {
    pub fn new() -> Self {
        let engine = windows::Media::Ocr::OcrEngine::TryCreateFromUserProfileLanguages().ok();
        Self { engine }
    }

    pub fn process_image(&self, img: image::RgbaImage) -> String {
        let engine = match &self.engine {
            Some(e) => e,
            None => return String::new(),
        };

        // 1. Convert RgbaImage to SoftwareBitmap via Stream
        let mut buf = Vec::new();
        let mut cursor = Cursor::new(&mut buf);
        DynamicImage::ImageRgba8(img).write_to(&mut cursor, image::ImageOutputFormat::Bmp).ok();
        
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

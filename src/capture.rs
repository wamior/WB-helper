use screenshots::Screen;

pub fn capture_fullscreen() -> Option<image::RgbaImage> {
    let screens = Screen::all().ok()?;
    let primary = screens.get(0)?; // Assuming primary for now
    primary.capture().ok()
}

pub fn capture_region(x: i32, y: i32, w: i32, h: i32) -> Option<image::RgbaImage> {
    let screens = Screen::all().ok()?;
    
    for screen in screens {
        let local_x = x - screen.display_info.x as i32;
        let local_y = y - screen.display_info.y as i32;
        
        if local_x >= 0 && local_x < screen.display_info.width as i32 &&
           local_y >= 0 && local_y < screen.display_info.height as i32 {
               return screen.capture_area(local_x, local_y, w as u32, h as u32).ok();
           }
    }
    None
}

use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
};

#[repr(C)]
#[allow(non_snake_case)]
pub struct DOCINFOW_MANUAL {
    pub cbSize: i32,
    pub lpszDocName: PCWSTR,
    pub lpszOutput: PCWSTR,
    pub lpszDatatype: PCWSTR,
    pub fwType: u32,
}

#[repr(C)]
#[allow(non_snake_case)]
#[derive(Clone, Copy)]
pub struct PRINTER_INFO_2W {
    pub pServerName: PWSTR,
    pub pPrinterName: PWSTR,
    pub pShareName: PWSTR,
    pub pPortName: PWSTR,
    pub pDriverName: PWSTR,
    pub pComment: PWSTR,
    pub pLocation: PWSTR,
    pub pDevMode: *mut std::ffi::c_void,
    pub pSepFile: PWSTR,
    pub pPrintProcessor: PWSTR,
    pub pDatatype: PWSTR,
    pub pParameters: PWSTR,
    pub pSecurityDescriptor: *mut std::ffi::c_void,
    pub Attributes: u32,
    pub Priority: u32,
    pub DefaultPriority: u32,
    pub StartTime: u32,
    pub UntilTime: u32,
    pub Status: u32,
    pub cJobs: u32,
    pub AveragePPM: u32,
}

// DEVMODE structure for printer configuration
#[repr(C)]
#[allow(non_snake_case)]
#[derive(Clone, Copy)]
pub struct DEVMODEW {
    pub dmDeviceName: [u16; 32],
    pub dmSpecVersion: u16,
    pub dmDriverVersion: u16,
    pub dmSize: u16,
    pub dmDriverExtra: u16,
    pub dmFields: u32,
    pub dmOrientation: i16,
    pub dmPaperSize: i16,
    pub dmPaperLength: i16,
    pub dmPaperWidth: i16,
    pub dmScale: i16,
    pub dmCopies: i16,
    pub dmDefaultSource: i16,
    pub dmPrintQuality: i16,
    pub dmColor: i16,
    pub dmDuplex: i16,
    pub dmYResolution: i16,
    pub dmTTOption: i16,
    pub dmCollate: i16,
    pub dmFormName: [u16; 32],
    pub dmLogPixels: u16,
    pub dmBitsPerPel: u32,
    pub dmPelsWidth: u32,
    pub dmPelsHeight: u32,
    pub dmDisplayFlags: u32,
    pub dmDisplayFrequency: u32,
    pub dmICMMethod: u32,
    pub dmICMIntent: u32,
    pub dmMediaType: u32,
    pub dmDitherType: u32,
    pub dmReserved1: u32,
    pub dmReserved2: u32,
    pub dmPanningWidth: u32,
    pub dmPanningHeight: u32,
}

// DEVMODE field constants
const DM_ORIENTATION: u32 = 0x00000001;
const DM_PAPERSIZE: u32 = 0x00000002;
const DM_PAPERLENGTH: u32 = 0x00000004;
const DM_PAPERWIDTH: u32 = 0x00000008;

// Paper size constants
const DMPAPER_USER: i16 = 256;

// Orientation constants
const DMORIENT_PORTRAIT: i16 = 1;
const DMORIENT_LANDSCAPE: i16 = 2;

#[link(name = "gdi32")]
extern "system" {
    pub fn StartDocW(hdc: HDC, lpdi: *const DOCINFOW_MANUAL) -> i32;
    pub fn StartPage(hdc: HDC) -> i32;
    pub fn EndPage(hdc: HDC) -> i32;
    pub fn EndDoc(hdc: HDC) -> i32;
}

#[link(name = "winspool")]
extern "system" {
    pub fn EnumPrintersW(
        flags: u32,
        name: PCWSTR,
        level: u32,
        p_printer_enum: *mut u8,
        cb_buf: u32,
        pcb_needed: *mut u32,
        pc_returned: *mut u32,
    ) -> BOOL;
    
    pub fn OpenPrinterW(
        pPrinterName: PCWSTR,
        phPrinter: *mut HANDLE,
        pDefault: *mut std::ffi::c_void,
    ) -> BOOL;
    
    pub fn ClosePrinter(hPrinter: HANDLE) -> BOOL;
    
    pub fn DocumentPropertiesW(
        hWnd: HWND,
        hPrinter: HANDLE,
        pDeviceName: PCWSTR,
        pDevModeOutput: *mut DEVMODEW,
        pDevModeInput: *const DEVMODEW,
        fMode: u32,
    ) -> i32;
}

const PRINTER_ENUM_LOCAL: u32 = 0x00000002;
const PRINTER_ENUM_CONNECTIONS: u32 = 0x00000004;

// DocumentProperties mode flags
const DM_OUT_BUFFER: u32 = 2;
const DM_IN_BUFFER: u32 = 8;

pub fn get_printers() -> Vec<String> {
    let mut printers = Vec::new();
    unsafe {
        let flags = PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS;
        let mut needed: u32 = 0;
        let mut returned: u32 = 0;

        let _ = EnumPrintersW(flags, PCWSTR::null(), 2, std::ptr::null_mut(), 0, &mut needed, &mut returned);
        
        if needed > 0 {
            let mut buffer = vec![0u8; needed as usize];
            if EnumPrintersW(
                flags,
                PCWSTR::null(),
                2,
                buffer.as_mut_ptr(),
                needed,
                &mut needed,
                &mut returned
            ).as_bool() {
                let p_info = buffer.as_ptr() as *const PRINTER_INFO_2W;
                for i in 0..returned {
                    let info = p_info.add(i as usize).read();
                    if !info.pPrinterName.is_null() {
                        if let Ok(name) = info.pPrinterName.to_string() {
                            printers.push(name);
                        }
                    }
                }
            }
        }
    }
    
    if printers.is_empty() {
        printers.push("Microsoft Print to PDF".to_string());
    }
    
    printers
}

pub fn print_label(printer_name: &str, text: &str, width_mm: f64, height_mm: f64, orientation: u8) -> anyhow::Result<()> {
    unsafe {
        let device_name_u16: Vec<u16> = if printer_name.is_empty() {
             return Err(anyhow::anyhow!("Printer name not specified"));
        } else {
             printer_name.encode_utf16().chain(Some(0)).collect()
        };
        
        // Open printer to get handle
        let mut h_printer = HANDLE::default();
        if !OpenPrinterW(
            PCWSTR(device_name_u16.as_ptr()),
            &mut h_printer,
            std::ptr::null_mut()
        ).as_bool() {
            return Err(anyhow::anyhow!("Failed to open printer: {}", printer_name));
        }
        
        // Get size of DEVMODE needed
        let size = DocumentPropertiesW(
            HWND::default(),
            h_printer,
            PCWSTR(device_name_u16.as_ptr()),
            std::ptr::null_mut(),
            std::ptr::null(),
            0
        );
        
        if size <= 0 {
            ClosePrinter(h_printer);
            return Err(anyhow::anyhow!("Failed to get DEVMODE size"));
        }
        
        // Allocate buffer for DEVMODE
        let mut buffer = vec![0u8; size as usize];
        let devmode_ptr = buffer.as_mut_ptr() as *mut DEVMODEW;
        
        // Get printer's default DEVMODE
        if DocumentPropertiesW(
            HWND::default(),
            h_printer,
            PCWSTR(device_name_u16.as_ptr()),
            devmode_ptr,
            std::ptr::null(),
            DM_OUT_BUFFER
        ) < 0 {
            ClosePrinter(h_printer);
            return Err(anyhow::anyhow!("Failed to get printer DEVMODE"));
        }
        
        // Modify DEVMODE with our settings
        let devmode = &mut *devmode_ptr;
        
        // Set fields that we're modifying
        devmode.dmFields |= DM_PAPERSIZE | DM_PAPERLENGTH | DM_PAPERWIDTH | DM_ORIENTATION;
        
        // Set custom paper size
        devmode.dmPaperSize = DMPAPER_USER;
        
        // Convert mm to 0.1mm units (tenths of a millimeter)
        // For landscape orientations, swap width and height in DEVMODE
        let (devmode_width, devmode_height) = if orientation == 1 || orientation == 3 {
            // Landscape: swap dimensions
            ((height_mm * 10.0) as i16, (width_mm * 10.0) as i16)
        } else {
            // Portrait: keep as is
            ((width_mm * 10.0) as i16, (height_mm * 10.0) as i16)
        };
        
        devmode.dmPaperWidth = devmode_width;
        devmode.dmPaperLength = devmode_height;
        
        // Set orientation (Windows only understands Portrait/Landscape, not 180° rotations)
        devmode.dmOrientation = if orientation == 1 || orientation == 3 {
            DMORIENT_LANDSCAPE
        } else {
            DMORIENT_PORTRAIT
        };
        
        // Apply the modified DEVMODE back to the printer
        if DocumentPropertiesW(
            HWND::default(),
            h_printer,
            PCWSTR(device_name_u16.as_ptr()),
            devmode_ptr,
            devmode_ptr,
            DM_IN_BUFFER | DM_OUT_BUFFER
        ) < 0 {
            ClosePrinter(h_printer);
            return Err(anyhow::anyhow!("Failed to apply DEVMODE settings"));
        }
        
        ClosePrinter(h_printer);
        
        let driver = "WINSPOOL\0".encode_utf16().collect::<Vec<u16>>();

        let hdc = CreateDCW(
            PCWSTR(driver.as_ptr()), 
            PCWSTR(device_name_u16.as_ptr()), 
            PCWSTR::null(), 
            Some(devmode_ptr as *const _)
        );
        
        if hdc.is_invalid() {
            return Err(anyhow::anyhow!("Failed to create printer DC: {}", printer_name));
        }

        let doc_name = "PVZ Label\0".encode_utf16().collect::<Vec<u16>>();
        
        let doc_info = DOCINFOW_MANUAL {
            cbSize: std::mem::size_of::<DOCINFOW_MANUAL>() as i32,
            lpszDocName: PCWSTR(doc_name.as_ptr()),
            lpszOutput: PCWSTR::null(),
            lpszDatatype: PCWSTR::null(), 
            fwType: 0,
        };

        if StartDocW(hdc, &doc_info) <= 0 {
            let _ = DeleteDC(hdc);
            return Err(anyhow::anyhow!("StartDoc failed"));
        }

        if StartPage(hdc) <= 0 {
            let _ = EndDoc(hdc);
            let _ = DeleteDC(hdc);
            return Err(anyhow::anyhow!("StartPage failed"));
        }

        let dpi_x = GetDeviceCaps(hdc, LOGPIXELSX);
        let dpi_y = GetDeviceCaps(hdc, LOGPIXELSY);
        
        // Calculate target dimensions based on original width/height (not swapped)
        let target_w_px = (width_mm / 25.4 * dpi_x as f64) as i32;
        let target_h_px = (height_mm / 25.4 * dpi_y as f64) as i32;
        
        // Apply 180° rotation if needed
        if orientation == 2 || orientation == 3 {
            // For 180° rotations, we need to transform the coordinate system
            SetGraphicsMode(hdc, GM_ADVANCED);
            
            let xform = XFORM {
                eM11: -1.0,
                eM12: 0.0,
                eM21: 0.0,
                eM22: -1.0,
                eDx: target_w_px as f32,
                eDy: target_h_px as f32,
            };
            SetWorldTransform(hdc, &xform);
        }
        
        let margin_x = (target_w_px as f64 * 0.05) as i32;
        let margin_y = (target_h_px as f64 * 0.05) as i32;
        
        let rect = RECT {
            left: margin_x,
            top: margin_y,
            right: target_w_px - margin_x,
            bottom: target_h_px - margin_y,
        };

        let mut font_height = 300;
        let min_font = 20;
        let font_name = "Arial\0".encode_utf16().collect::<Vec<u16>>();
        
        loop {
            if font_height < min_font { break; }
            
            let hfont = CreateFontW(
                font_height, 0, 0, 0, FW_BOLD.0 as i32, 
                0, 0, 0, 
                DEFAULT_CHARSET.0 as u32,
                OUT_DEFAULT_PRECIS.0 as u32, 
                CLIP_DEFAULT_PRECIS.0 as u32, 
                DEFAULT_QUALITY.0 as u32, 
                DEFAULT_PITCH.0 as u32, 
                PCWSTR(font_name.as_ptr())
            );
            
            let old_obj = SelectObject(hdc, hfont);
            let mut size = SIZE::default();
            let text_u16: Vec<u16> = text.encode_utf16().collect(); 
            
            let _ = GetTextExtentPoint32W(hdc, &text_u16, &mut size);
            
            let w_avail = rect.right - rect.left;
            let h_avail = rect.bottom - rect.top;
            
            if size.cx <= w_avail && size.cy <= h_avail {
                 let x_center = rect.left + (w_avail - size.cx) / 2;
                 let y_center = rect.top + (h_avail - size.cy) / 2;
                 
                 let _ = TextOutW(hdc, x_center, y_center, &text_u16);
                 
                 SelectObject(hdc, old_obj);
                 let _ = DeleteObject(hfont);
                 break;
            }
            
            SelectObject(hdc, old_obj);
            let _ = DeleteObject(hfont);
            
             if size.cx > w_avail * 2 { font_height -= 50; }
             else if size.cx > (w_avail as f64 * 1.5) as i32 { font_height -= 20; }
             else { font_height -= 5; }
        }

        EndPage(hdc);
        EndDoc(hdc);
        let _ = DeleteDC(hdc);
    }
    Ok(())
}

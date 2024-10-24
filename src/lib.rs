// Wokwi Custom Chips with Rust
//
// Very rough prototype by Uri Shaked
//
// Look at chipInit() at the bottom, and open Chrome devtools console to see the debugPrint().

use std::ffi::{c_void, CString};

use wokwi_chip_ll::{
    bufferWrite, debugPrint, framebufferInit, pinInit, pinWatch, timerInit, timerStart, BufferId,
    PinId, TimerConfig, WatchConfig, INPUT, RISING,
};

//use reqwest::get;
//use image::{load_from_memory, ImageFormat};
//use image::load_from_memory;

const MS: u32 = 1000; // micros

// Colors:
const DEEP_GREEN: u32 = 0xff003000; // ARGB
const PURPLE: u32 = 0xffff00ff; // ARGB

struct Chip {
    frame_buffer: BufferId,
    width: u32,
    height: u32,
    current_row: u32,
    ra: f32,
    dec: f32,
    ra_move: f32,
    dec_move: f32,
    pin_ra_dir: PinId,
    pin_ra_step: PinId,
    pin_dec_dir: PinId,
    pin_dec_step: PinId,
}

// chipInit() will be called once per chip instance. We use CHIP_VEC to keep track of all the
// instances, and use the user_data pointer to index into CHIP_VEC.
static mut CHIP_VEC: Vec<Chip> = Vec::new();

fn draw_line(chip: &Chip, row: u32, color: u32) {
    let color_bytes_ptr = &color as *const u32 as *const u8;

    let offset = chip.width * 4 * row;
    for x in (0..chip.width * 4).step_by(4) {
        unsafe {
            bufferWrite(chip.frame_buffer, offset + x, color_bytes_ptr, 4);
        }
    }
}

pub unsafe fn on_pin_ra_step_change(user_data: *const c_void, _pin: PinId, value: u32) {
    let mut chip = &mut CHIP_VEC[user_data as usize];
    /*
    if value == HIGH {
        pinWrite(chip.pin_out, LOW);
    } else {
        pinWrite(chip.pin_out, HIGH);
    } */
    chip.ra = chip.ra + chip.ra_move;
    debugPrint(CString::new("Ha cambiado ra step").unwrap().into_raw());
}

pub unsafe fn on_pin_ra_dir_change(user_data: *const c_void, _pin: PinId, value: u32) {
    let mut chip = &mut CHIP_VEC[user_data as usize];
    /*
    if value == HIGH {
        pinWrite(chip.pin_out, LOW);
    } else {
        pinWrite(chip.pin_out, HIGH);
    } */
    chip.ra_move = -1.0 * chip.ra_move;
    debugPrint(CString::new("Ha cambiado ra dir").unwrap().into_raw());
}

pub unsafe fn on_pin_dec_step_change(user_data: *const c_void, _pin: PinId, value: u32) {
    let mut chip = &mut CHIP_VEC[user_data as usize];
    /*
    if value == HIGH {
        pinWrite(chip.pin_out, LOW);
    } else {
        pinWrite(chip.pin_out, HIGH);
    } */
    chip.dec = chip.dec + chip.dec_move;
    debugPrint(CString::new("Ha cambiado dec step").unwrap().into_raw());
}

pub unsafe fn on_pin_dec_dir_change(user_data: *const c_void, _pin: PinId, value: u32) {
    let mut chip = &mut CHIP_VEC[user_data as usize];
    /*
    if value == HIGH {
        pinWrite(chip.pin_out, LOW);
    } else {
        pinWrite(chip.pin_out, HIGH);
    } */
    chip.dec_move = -1.0 * chip.dec_move;
    debugPrint(CString::new("Ha cambiado dec dir").unwrap().into_raw());
}

// TODO: separar el fetch del volcado de la imagen
pub async unsafe fn fetch_image(
    user_data: *const c_void,
) -> Result<u32, Box<dyn std::error::Error>> {
    let mut chip = &mut CHIP_VEC[user_data as usize];

    let url = format!(
        "https://skyserver.sdss.org/dr12/SkyserverWS/ImgCutout/getjpeg?TaskName=Skyserver.Chart.Image&ra={}&dec={}&scale=80&width=128&height=128&opt=&query=",
        chip.ra, chip.dec
    );
    /*
    let res = reqwest::get(url)
        .await
        .expect("failed to get response")
        .bytes()
        .await
        .expect("failed to get payload");
    */
    /*
    let body = load_from_memory(&res)?.to_rgba8();
    let img = body.as_raw();

    unsafe {
        bufferWrite(chip.frame_buffer, 0, img.as_ptr(), 4 * 128 * 128);
    }
    */

    let response = minreq::get(url).send()?;
    assert_eq!(200, response.status_code);
    let body = response.as_bytes();

    /*
    let img = image::load_from_memory(&body)?.to_rgba8();
    let rgba = img.as_raw();
    unsafe {
        bufferWrite(chip.frame_buffer, 0, img.as_ptr(), 4 * 128 * 128);
    }
    */
    let mut options = zune_core::options::DecoderOptions::default()
        .jpeg_set_out_colorspace(zune_core::colorspace::ColorSpace::RGBA);
    let mut decoder = zune_jpeg::JpegDecoder::new_with_options(body, options);
    let pixels = decoder.decode()?;
    unsafe {
        bufferWrite(chip.frame_buffer, 0, pixels.as_ptr(), 4 * 128 * 128);
    }
    Ok(0)
}

pub unsafe fn on_timer_fired(user_data: *const c_void) {
    //let mut chip = &mut CHIP_VEC[user_data as usize];

    fetch_image(user_data);
    /*
    match fetch_image(user_data) {
        Ok(0) => {
            // Aquí puedes usar `img` que es de tipo `image::RgbaImage`
            // println!("Imagen descargada y convertida con éxito.");
            // debugPrint(CString::new("Volcamos al FB la imagen").unwrap().into_raw());
            // unsafe {
            //     bufferWrite(chip.frame_buffer, 0, img, 4 * 128 * 128);
            // }
            debugPrint(CString::new("imagen volcada al FB").unwrap().into_raw());
        }
        Err(e) => {
            debugPrint(CString::new("Error al descargar o procesar la imagen").unwrap().into_raw());
            // eprintln!("Error al descargar o procesar la imagen: {}", e);
        }
    }
    */
}

#[no_mangle]
pub unsafe extern "C" fn chipInit() {
    debugPrint(
        CString::new("Hello from WokwiScope Chip!")
            .unwrap()
            .into_raw(),
    );

    let mut width = 0;
    let mut height = 0;
    let frame_buffer = framebufferInit(&mut width, &mut height);

    let chip = Chip {
        frame_buffer,
        width,
        height,
        current_row: 0,
        //ra: 0.0,
        //dec: 0.0,
        ra: 83.8245,
        dec: -4.6114,
        ra_move: 0.9,
        dec_move: 0.9,
        pin_ra_dir: pinInit(CString::new("RA-DIR").unwrap().into_raw(), INPUT),
        pin_ra_step: pinInit(CString::new("RA-STEP").unwrap().into_raw(), INPUT),
        pin_dec_dir: pinInit(CString::new("DEC-DIR").unwrap().into_raw(), INPUT),
        pin_dec_step: pinInit(CString::new("DEC-STEP").unwrap().into_raw(), INPUT),
    };
    CHIP_VEC.push(chip);

    let timer_config = TimerConfig {
        user_data: (CHIP_VEC.len() - 1) as *const c_void,
        callback: on_timer_fired as *const c_void,
    };

    let timer = timerInit(&timer_config);
    timerStart(timer, 60000 * MS, true);

    let watch_config = WatchConfig {
        user_data: (CHIP_VEC.len() - 1) as *const c_void,
        edge: RISING,
        pin_change: on_pin_ra_dir_change as *const c_void,
    };

    let chip = CHIP_VEC.last().unwrap();

    pinWatch(chip.pin_ra_dir, &watch_config);

    let watch_config = WatchConfig {
        user_data: (CHIP_VEC.len() - 1) as *const c_void,
        edge: RISING,
        pin_change: on_pin_ra_step_change as *const c_void,
    };

    pinWatch(chip.pin_ra_step, &watch_config);

    let watch_config = WatchConfig {
        user_data: (CHIP_VEC.len() - 1) as *const c_void,
        edge: RISING,
        pin_change: on_pin_dec_dir_change as *const c_void,
    };

    pinWatch(chip.pin_dec_dir, &watch_config);

    let watch_config = WatchConfig {
        user_data: (CHIP_VEC.len() - 1) as *const c_void,
        edge: RISING,
        pin_change: on_pin_dec_step_change as *const c_void,
    };

    pinWatch(chip.pin_dec_step, &watch_config);
}

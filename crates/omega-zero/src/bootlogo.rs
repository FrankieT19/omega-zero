use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use omega_ui::UndoFace;

const LOGO: &str = "bootlogo.bmp";
const ORIGINAL: &str = "bootlogo.baseos.bmp";
const PARTITION: &str = "boot-resource";
const MOUNT: &str = "/tmp/omega-boot-resource";
const HEADER: usize = 54;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Outcome {
    Same,
    Installed,
    Invalid,
    /// The logo is not the size of the one already on the partition, which is the size the
    /// bootloader draws on this device's panel.
    WrongSize,
}

/// The width and height of an uncompressed, bottom-up, 24-bit bmp, the only kind the
/// bootloader draws.
pub fn dims(bmp: &[u8]) -> Option<(u32, u32)> {
    if bmp.len() < HEADER || &bmp[0..2] != b"BM" {
        return None;
    }
    let u16_at = |i: usize| u16::from_le_bytes([bmp[i], bmp[i + 1]]);
    let u32_at = |i: usize| u32::from_le_bytes([bmp[i], bmp[i + 1], bmp[i + 2], bmp[i + 3]]);
    let (w, h) = (u32_at(18), u32_at(22));
    let well_formed = u32_at(10) == HEADER as u32
        && u32_at(14) == 40
        && u16_at(26) == 1
        && u16_at(28) == 24
        && u32_at(30) == 0
        && (1..=4096).contains(&w)
        && (1..=4096).contains(&h);
    let stride = (w as usize * 3).next_multiple_of(4);
    (well_formed && bmp.len() == HEADER + stride * h as usize).then_some((w, h))
}

pub fn valid(bmp: &[u8]) -> bool {
    dims(bmp).is_some()
}

/// The picture in a bmp the bootloader draws, top row first, four bytes a pixel and opaque, as
/// a face to upload.
pub fn face(bmp: &[u8]) -> Option<UndoFace> {
    let (w, h) = dims(bmp)?;
    let (cols, rows) = (w as usize, h as usize);
    let stride = (cols * 3).next_multiple_of(4);
    let mut rgba = Vec::with_capacity(cols * rows * 4);
    for y in (0..rows).rev() {
        let row = &bmp[HEADER + y * stride..][..cols * 3];
        for bgr in row.chunks_exact(3) {
            rgba.extend_from_slice(&[bgr[2], bgr[1], bgr[0], 0xff]);
        }
    }
    Some(UndoFace { rgba, w, h })
}

/// The logo the bootloader draws on a panel of `size`, read from the card's System folder: the
/// one `refresh` keeps on the boot partition for that panel.
pub fn shown(system: &Path, size: (u32, u32)) -> Option<UndoFace> {
    let logo = face(&logo_bytes(system, size)?)?;
    ((logo.w, logo.h) == size).then_some(logo)
}

/// Converts the replaceable PNG to the bootloader's native BMP in memory.
/// Letterboxing keeps arbitrary replacement images in proportion on each panel.
pub fn logo_bytes(system: &Path, size: (u32, u32)) -> Option<Vec<u8>> {
    if let Some(bmp) = png_bmp(&system.join("bootlogo.png"), size) {
        return Some(bmp);
    }
    fs::read(choose(system, size)?).ok()
}

fn png_bmp(path: &Path, (w, h): (u32, u32)) -> Option<Vec<u8>> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(fs::read(path).ok()?));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
    let mut pixels = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut pixels).ok()?;
    let (sw, sh) = (info.width, info.height);
    if sw == 0 || sh == 0 || w == 0 || h == 0 {
        return None;
    }
    let channels = info.color_type.samples();
    let scale = (w as f64 / sw as f64).min(h as f64 / sh as f64);
    let (dw, dh) = (
        (sw as f64 * scale).round() as u32,
        (sh as f64 * scale).round() as u32,
    );
    if dw == 0 || dh == 0 {
        return None;
    }
    let stride = (w as usize * 3).next_multiple_of(4);
    let mut bmp = vec![0; HEADER + stride * h as usize];
    bmp[..2].copy_from_slice(b"BM");
    let len = bmp.len() as u32;
    for (at, value) in [(2, len), (10, HEADER as u32), (14, 40), (18, w), (22, h)] {
        bmp[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    bmp[26..28].copy_from_slice(&1u16.to_le_bytes());
    bmp[28..30].copy_from_slice(&24u16.to_le_bytes());
    for y in 0..dh {
        for x in 0..dw {
            let at = ((y as u64 * sh as u64 / dh as u64) * sw as u64
                + x as u64 * sw as u64 / dw as u64) as usize
                * channels;
            let c = &pixels[at..at + channels];
            let (rgb, alpha) = match info.color_type {
                png::ColorType::Rgb => ([c[0], c[1], c[2]], 255),
                png::ColorType::Rgba => ([c[0], c[1], c[2]], c[3]),
                png::ColorType::Grayscale => ([c[0]; 3], 255),
                png::ColorType::GrayscaleAlpha => ([c[0]; 3], c[1]),
                _ => return None,
            };
            let dest = HEADER
                + (h - 1 - y - (h - dh) / 2) as usize * stride
                + (x + (w - dw) / 2) as usize * 3;
            for i in 0..3 {
                bmp[dest + i] = (rgb[2 - i] as u16 * alpha as u16 / 255) as u8;
            }
        }
    }
    Some(bmp)
}

/// The logo omega-zero ships for a panel of `size`, from the card's System folder: the one named for
/// that size, or the unsized one from before omega-zero knew more than one panel, if it fits.
pub fn choose(system: &Path, size: (u32, u32)) -> Option<PathBuf> {
    let sized = system.join(format!("bootlogo-{}x{}.bmp", size.0, size.1));
    if sized.is_file() {
        return Some(sized);
    }
    let legacy = system.join(LOGO);
    let fits = fs::read(&legacy).ok().and_then(|b| dims(&b)) == Some(size);
    fits.then_some(legacy)
}

/// The panel size the bootloader on this partition draws at, read off the logo it came with.
pub fn panel_size(dir: &Path) -> Option<(u32, u32)> {
    [ORIGINAL, LOGO]
        .iter()
        .find_map(|name| fs::read(dir.join(name)).ok().and_then(|b| dims(&b)))
}

pub fn install(logo: &[u8], dir: &Path) -> io::Result<Outcome> {
    let Some(size) = dims(logo) else {
        return Ok(Outcome::Invalid);
    };
    if panel_size(dir) != Some(size) {
        return Ok(Outcome::WrongSize);
    }
    let current = fs::read(dir.join(LOGO)).ok();
    if current.as_deref() == Some(logo) {
        return Ok(Outcome::Same);
    }
    if let Some(current) = &current {
        if !dir.join(ORIGINAL).exists() {
            replace(dir, ORIGINAL, current)?;
        }
    }
    replace(dir, LOGO, logo)?;
    Ok(Outcome::Installed)
}

fn replace(dir: &Path, name: &str, bytes: &[u8]) -> io::Result<()> {
    let tmp = dir.join("bootlogo.tmp");
    let mut f = File::create(&tmp)?;
    f.write_all(bytes)?;
    f.sync_all()?;
    drop(f);
    fs::rename(&tmp, dir.join(name))?;
    File::open(dir)?.sync_all()
}

fn ships_a_logo(system: &Path) -> bool {
    system.join("bootlogo.png").is_file()
        || fs::read_dir(system).is_ok_and(|entries| {
            entries.flatten().any(|e| {
                let name = e.file_name();
                let name = name.to_string_lossy();
                name.starts_with("bootlogo") && name.ends_with(".bmp")
            })
        })
}

pub fn refresh(card: &Path) {
    let system = card.join("System");
    if !ships_a_logo(&system) {
        return;
    }
    let Some(device) = partition(PARTITION) else {
        eprintln!("omega-zero: bootlogo: no {PARTITION} partition");
        return;
    };
    if fs::create_dir_all(MOUNT).is_err()
        || !run("mount", &["-t", "vfat", "-o", "rw,noatime", &device, MOUNT])
    {
        eprintln!("omega-zero: bootlogo: could not mount {device}");
        return;
    }
    let mount = Path::new(MOUNT);
    match panel_size(mount) {
        None => {
            eprintln!("omega-zero: bootlogo: no logo on {device} to size ours by, leaving it alone")
        }
        Some(size) => match logo_bytes(&system, size) {
            None => eprintln!("omega-zero: bootlogo: no valid logo in System/"),
            Some(logo) => match install(&logo, mount) {
                Ok(Outcome::Installed) => {
                    eprintln!("omega-zero: bootlogo: installed System/bootlogo.png")
                }
                Ok(Outcome::Same) => {}
                Ok(other) => eprintln!("omega-zero: bootlogo: {other:?}"),
                Err(e) => eprintln!("omega-zero: bootlogo: {e}"),
            },
        },
    }
    if !run("umount", &[MOUNT]) {
        eprintln!("omega-zero: bootlogo: could not unmount {MOUNT}");
    }
}

fn partition(name: &str) -> Option<String> {
    for entry in fs::read_dir("/sys/class/block").ok()?.flatten() {
        let uevent = fs::read_to_string(entry.path().join("uevent")).unwrap_or_default();
        if uevent.lines().any(|l| l == format!("PARTNAME={name}")) {
            return Some(
                PathBuf::from("/dev")
                    .join(entry.file_name())
                    .display()
                    .to_string(),
            );
        }
    }
    None
}

fn run(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .status()
        .is_ok_and(|s| s.success())
}

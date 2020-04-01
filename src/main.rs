extern crate edid;

use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;

use edid::Descriptor;
use edid::EDID;

const EXE_NAME: &str = "redid-parse";
const MAX_EDID_SIZE: usize = 4096;

fn usage() -> ! {
    eprintln!("Usage: {} <path>'", EXE_NAME);
    std::process::exit(1);
}

fn bits_per_pixel(video_input: u8) -> Option<usize> {
    match video_input & 0x70u8 {
        0x0 => Some(0),
        0x10 => Some(6),
        0x20 => Some(8),
        0x30 => Some(10),
        0x40 => Some(12),
        0x50 => Some(14),
        0x60 => Some(16),
        _ => None,
    }
}

fn video_interface(video_input: u8) -> Option<&'static str> {
    match video_input & 0xfu8 {
        0 => Some("Undefined"),
        2 => Some("HDMIa"),
        3 => Some("HDMIb"),
        4 => Some("MDDI"),
        5 => Some("DisplayPort"),
        _ => None,
    }
}

fn video_white_sync_levels(video_input: u8) -> &'static str {
    match video_input & 0x60 {
        0x0u8 => "+0.7/−0.3 V",
        0x10u8 => "+0.714/−0.286 V",
        0x20u8 => "+1.0/−0.4 V",
        0x30u8 => "+0.7/0 V",
        _ => "",
    }
}

fn analog_display_type(features: u8) -> &'static str {
    match features & 0x18 {
        0x0 => "Monochrome or grayscale",
        0x8 => "RGB color",
        0x10 => "Non-RGB color",
        _ => "Undefined",
    }
}

fn supported(v: bool) -> &'static str {
    match v {
        true => "Supported",
        false => "Unsupported",
    }
}

fn yes_or_no(v: bool) -> &'static str {
    match v {
        true => "Yes",
        false => "No",
    }
}

fn pretty_print(e: &EDID, raw: bool) {
    println!("Header:");
    println!("  Year: {}", 1990 + e.header.year as u32);
    println!("  Week: {}", e.header.week);
    println!("  Product: {:04x}", e.header.product);
    println!("  Serial: {:08x}", e.header.serial);
    println!("  Version: {}.{}", e.header.version, e.header.revision);
    println!("");

    println!("Display:");
    println!("  Size: {}x{} cm", e.display.width, e.display.height);
    let video_input = e.display.video_input;
    let features = e.display.features;
    if video_input & (1u8 << 7) != 0 {
        println!("  Type: Digital");
        match bits_per_pixel(video_input) {
            Some(0) => println!("  Bits depth: Undefined"),
            Some(v) => println!("  Bits depth: {} bpp", v),
            None => println!("  Bits depth: Unknown ({})", video_input & 0x70u8),
        }
        match video_interface(video_input) {
            Some(v) => println!("  Video interface: {}", v),
            None => println!("  Video interface: Unknown ({})", video_input & 0xfu8),
        }
        println!("  YCrCb 4:4:4: {}", supported((features & (1u8 << 3)) != 0));
        println!("  YCrCb 4:2:2: {}", supported((features & (1u8 << 4)) != 0));
    } else {
        println!("  Type: Analog");
        println!(
            "  Video white and sync levels: {}",
            video_white_sync_levels(video_input)
        );
        println!(
            "  Blank-to-black setup (pedestal): {}",
            if video_input & (1u8 << 4) != 0 {
                "Expected"
            } else {
                "Not set"
            }
        );
        println!(
            "  Separate sync: {}",
            supported((video_input & (1u8 << 3)) != 0)
        );
        println!(
            "  Composite sync: {}",
            supported((video_input & (1u8 << 2)) != 0)
        );
        println!(
            "  Sync on green: {}",
            supported((video_input & (1u8 << 1)) != 0)
        );
        println!(
            "  VSync pulse must be serrated: {}",
            yes_or_no((video_input & (1u8 << 0)) != 0)
        );
        println!("  Display type: {}", analog_display_type(features));
    }
    println!("  Standby: {}", supported(features & (1u8 << 7) != 0));
    println!("  Suspend: {}", supported(features & (1u8 << 6) != 0));
    println!("  Active-off: {}", supported(features & (1u8 << 5) != 0));
    if raw {
        println!("");
        println!("  Video input: {:04x} ({:08b})", video_input, video_input);
        println!("  Features: {:04x} ({:08b})", features, features);
    }
    println!("");
    println!("Descriptors:");
    for d in &e.descriptors {
        match d {
            Descriptor::Dummy => {},
            Descriptor::DetailedTiming(_) => println!("  DetailedTiming"),
            Descriptor::SerialNumber(s) => println!("  Serial Number: {}", s),
            Descriptor::UnspecifiedText(s) => println!("  Text: {}", s),
            Descriptor::RangeLimits => println!("  RangeLimits"),
            Descriptor::ProductName(s) => println!("  ProductName: {}", s),
            Descriptor::WhitePoint => println!("  WhitePoint"),
            Descriptor::StandardTiming => println!("  StandardTiming"),
            Descriptor::ColorManagement => println!("  ColorManagement"),
            Descriptor::TimingCodes => println!("  TimingCodes"),
            Descriptor::EstablishedTimings => println!("  EstablishedTimings"),
            Descriptor::Unknown(b) => println!("  Unknown: {:04x}", b[0]),
        }
    }
}

fn parse(path: &OsStr) {
    let mut f = File::open(path).unwrap();
    let mut raw = [0u8; MAX_EDID_SIZE];
    f.read(&mut raw).unwrap();
    let (_, e) = edid::parse(&raw).unwrap();
    println!("{}:", path.to_str().unwrap());
    pretty_print(&e, true);
}

fn main() {
    match std::env::args_os().len() {
        2 => parse(&env::args_os().collect::<Vec<_>>()[1]),
        _ => usage(),
    }
}

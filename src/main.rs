extern crate edid;

use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;

use edid::Descriptor;
use edid::*;

const EXE_NAME: &str = "redid-parse";
const MAX_EDID_SIZE: usize = 4096;
const SHIFT: usize = 2;

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

fn audio_format_string(v: u8) -> &'static str {
    match v {
        ShortAudioDescriptor::LPCM => "LPCM",
        ShortAudioDescriptor::AC3 => "AC3",
        ShortAudioDescriptor::MPEG1 => "MPEG1",
        ShortAudioDescriptor::MP3 => "MP3",
        ShortAudioDescriptor::MPEG2 => "MPEG2",
        ShortAudioDescriptor::AAC => "AAC",
        ShortAudioDescriptor::DTS => "DTS",
        ShortAudioDescriptor::ATRAC => "ATRAC",
        ShortAudioDescriptor::DSD => "DSD",
        ShortAudioDescriptor::DDPLUS => "DD+",
        ShortAudioDescriptor::DTSHD => "DTS-HD",
        ShortAudioDescriptor::TRUEHD => "Dolby TrueHD",
        ShortAudioDescriptor::DSTAUDIO => "DST Audio",
        ShortAudioDescriptor::WMAPRO => "WMA Pro",
        _ => "Unknown",
    }
}

fn print_dtd(depth: usize, dt: &DetailedTiming) {
    println!("{:1$}Detailed timing:", "", SHIFT * depth);
    println!(
        "{:1$}Resolution: {2}x{3}",
        "",
        SHIFT * (depth + 1),
        dt.horizontal_active_pixels,
        dt.vertical_active_lines
    );
    println!(
        "{:1$}Size: {2}x{3} mm",
        "",
        SHIFT * (depth + 1),
        dt.horizontal_size,
        dt.vertical_size
    );
}

fn pretty_print(e: &EDID, raw: bool) {
    let depth = 1;
    println!("Header:");
    println!(
        "{:1$}Year: {2}",
        "",
        SHIFT * depth,
        1990 + e.header.year as u32
    );
    println!("{:1$}Week: {2}", "", SHIFT * depth, e.header.week);
    println!("{:1$}Product: {2:04x}", "", SHIFT * depth, e.header.product);
    println!("{:1$}Serial: {2:08x}", "", SHIFT * depth, e.header.serial);
    println!(
        "{:1$}Version: {2}.{3}",
        "",
        SHIFT * depth,
        e.header.version,
        e.header.revision
    );
    println!("");

    println!("Display:");
    println!(
        "{:1$}Size: {2}x{3} cm",
        "",
        SHIFT * depth,
        e.display.width,
        e.display.height
    );
    let video_input = e.display.video_input;
    let features = e.display.features;
    if video_input & (1u8 << 7) != 0 {
        println!("{:1$}Type: Digital", "", SHIFT * depth);
        match bits_per_pixel(video_input) {
            Some(0) => println!("{:1$}Bits depth: Undefined", "", SHIFT * depth),
            Some(v) => println!("{:1$}Bits depth: {2} bpp", "", SHIFT * depth, v),
            None => println!(
                "{:1$}Bits depth: Unknown ({2})",
                "",
                SHIFT * depth,
                video_input & 0x70u8
            ),
        }
        match video_interface(video_input) {
            Some(v) => println!("{:1$}Video interface: {2}", "", SHIFT * depth, v),
            None => println!(
                "{:1$}Video interface: Unknown ({2})",
                "",
                SHIFT * depth,
                video_input & 0xfu8
            ),
        }
        println!(
            "{:1$}YCrCb 4:4:4: {2}",
            "",
            SHIFT * depth,
            supported((features & (1u8 << 3)) != 0)
        );
        println!(
            "{:1$}YCrCb 4:2:2: {2}",
            "",
            SHIFT * depth,
            supported((features & (1u8 << 4)) != 0)
        );
    } else {
        println!("{:1$}Type: Analog", "", SHIFT * depth);
        println!(
            "{:1$}Video white and sync levels: {2}",
            "",
            SHIFT * depth,
            video_white_sync_levels(video_input)
        );
        println!(
            "{:1$}Blank-to-black setup (pedestal): {2}",
            "",
            SHIFT * depth,
            if video_input & (1u8 << 4) != 0 {
                "Expected"
            } else {
                "Not set"
            }
        );
        println!(
            "{:1$}Separate sync: {2}",
            "",
            SHIFT * depth,
            supported((video_input & (1u8 << 3)) != 0)
        );
        println!(
            "{:1$}Composite sync: {2}",
            "",
            SHIFT * depth,
            supported((video_input & (1u8 << 2)) != 0)
        );
        println!(
            "{:1$}Sync on green: {2}",
            "",
            SHIFT * depth,
            supported((video_input & (1u8 << 1)) != 0)
        );
        println!(
            "{:1$}VSync pulse must be serrated: {2}",
            "",
            SHIFT * depth,
            yes_or_no((video_input & (1u8 << 0)) != 0)
        );
        println!(
            "{:1$}Display type: {2}",
            "",
            SHIFT * depth,
            analog_display_type(features)
        );
    }
    println!(
        "{:1$}Standby: {2}",
        "",
        SHIFT * depth,
        supported(features & (1u8 << 7) != 0)
    );
    println!(
        "{:1$}Suspend: {2}",
        "",
        SHIFT * depth,
        supported(features & (1u8 << 6) != 0)
    );
    println!(
        "{:1$}Active-off: {2}",
        "",
        SHIFT * depth,
        supported(features & (1u8 << 5) != 0)
    );
    if raw {
        println!("");
        println!(
            "{:1$}Video input: {2:02x} ({2:08b})",
            "",
            SHIFT * depth,
            video_input
        );
        println!(
            "{:1$}Features: {2:02x} ({2:08b})",
            "",
            SHIFT * depth,
            features
        );
    }
    println!("");
    println!("Descriptors:");
    for d in &e.descriptors {
        match d {
            Descriptor::Dummy => {}
            Descriptor::DetailedTiming(dtd) => print_dtd(depth, dtd),
            Descriptor::SerialNumber(s) => {
                println!("{:1$}Serial Number: {2}", "", SHIFT * depth, s)
            }
            Descriptor::UnspecifiedText(s) => println!("{:1$}Text: {2}", "", SHIFT * depth, s),
            Descriptor::RangeLimits => println!("{:1$}RangeLimits", "", SHIFT * depth),
            Descriptor::ProductName(s) => println!("{:1$}ProductName: {2}", "", SHIFT * depth, s),
            Descriptor::WhitePoint => println!("{:1$}WhitePoint", "", SHIFT * depth),
            Descriptor::StandardTiming => println!("{:1$}StandardTiming", "", SHIFT * depth),
            Descriptor::ColorManagement => println!("{:1$}ColorManagement", "", SHIFT * depth),
            Descriptor::TimingCodes => println!("{:1$}TimingCodes", "", SHIFT * depth),
            Descriptor::EstablishedTimings => {
                println!("{:1$}EstablishedTimings", "", SHIFT * depth)
            }
            Descriptor::Unknown(b) => println!("{:1$}Unknown: {2:04x}", "", SHIFT * depth, b[0]),
        }
    }
    if e.extension.is_none() {
        return;
    }
    let x = e.extension.as_ref().unwrap();
    println!("");
    println!("Extension:");
    println!(
        "{:1$}Underscan: {2}",
        "",
        SHIFT * depth,
        supported((x.native_dtd & edid::CEAEDID::DTD_UNDERSCAN) != 0)
    );
    println!(
        "{:1$}Basic audio: {2}",
        "",
        SHIFT * depth,
        supported((x.native_dtd & edid::CEAEDID::DTD_BASIC_AUDIO) != 0)
    );
    println!(
        "{:1$}YCbCr 4∶4∶4: {2}",
        "",
        SHIFT * depth,
        supported((x.native_dtd & edid::CEAEDID::DTD_YUV444) != 0)
    );
    println!(
        "{:1$}YCbCr 4∶2∶2: {2}",
        "",
        SHIFT * depth,
        supported((x.native_dtd & edid::CEAEDID::DTD_YUV422) != 0)
    );
    if raw {
        println!("{:1$}native_dtd: {2:08b}", "", SHIFT * depth, x.native_dtd);
    }
    if x.blocks.len() > 0 {
        println!("");
        println!("Blocks:");
        for b in &x.blocks {
            match b {
                DataBlock::AudioBlock(v) => {
                    println!("{:1$}Supported audio formats:", "", SHIFT * depth);
                    for a in &v.descriptors {
                        let format = a.format();
                        if format == 0 || format == ShortAudioDescriptor::RESERVED {
                            continue;
                        }
                        let bd = a.bit_depths().unwrap_or_default();
                        let b16 = if bd & ShortAudioDescriptor::LPCM_16_BIT > 0 {
                            " 16 bit"
                        } else {
                            ""
                        };
                        let b20 = if bd & ShortAudioDescriptor::LPCM_20_BIT > 0 {
                            " 20 bit"
                        } else {
                            ""
                        };
                        let b24 = if bd & ShortAudioDescriptor::LPCM_24_BIT > 0 {
                            " 24 bit"
                        } else {
                            ""
                        };
                        let bitrate = if let Some(v) = a.bitrate() {
                            format!(" max bitrate {} kbps", v)
                        } else {
                            String::new()
                        };
                        println!(
                            "{:1$}{2} {3} channels{4}{5}{6}{7}",
                            "",
                            SHIFT * (depth + 1),
                            audio_format_string(format),
                            a.channels(),
                            bitrate,
                            b16,
                            b20,
                            b24
                        );
                    }
                }

                DataBlock::VideoBlock(v) => {
                    println!("{:1$}Supported video formats:", "", SHIFT * depth);
                    for d in &v.descriptors {
                        println!(
                            "{:1$}{2}{3}",
                            "",
                            SHIFT * (depth + 1),
                            d.cea861_index(),
                            if d.is_native() { " (native)" } else { "" }
                        );
                    }
                }

                DataBlock::VendorSpecific(v) => {
                    println!(
                        "{:1$}Vendor specific: {2:02x} {3:02x} {4:02x}",
                        "",
                        SHIFT * depth,
                        v.identifier[0],
                        v.identifier[1],
                        v.identifier[2]
                    );
                }

                DataBlock::SpeakerAllocation(v) => {
                    println!(
                        "{:1$}Speaker allocation:{2}{3}{4}{5}{6}{7}{8}",
                        "",
                        SHIFT * depth,
                        if v.speakers & SpeakerAllocation::FRONT_LEFT_RIGHT > 0 {
                            " FL FR"
                        } else {
                            ""
                        },
                        if v.speakers & SpeakerAllocation::LFE > 0 {
                            " LFE"
                        } else {
                            ""
                        },
                        if v.speakers & SpeakerAllocation::FRONT_CENTER > 0 {
                            " FC"
                        } else {
                            ""
                        },
                        if v.speakers & SpeakerAllocation::REAR_LEFT_RIGHT > 0 {
                            " RL RR"
                        } else {
                            ""
                        },
                        if v.speakers & SpeakerAllocation::REAR_CENTER > 0 {
                            " RC"
                        } else {
                            ""
                        },
                        if v.speakers & SpeakerAllocation::FRONT_LEFT_RIGHT_CENTER > 0 {
                            " FLRC"
                        } else {
                            ""
                        },
                        if v.speakers & SpeakerAllocation::REAR_LEFT_RIGHT_CENTER > 0 {
                            " RLRC"
                        } else {
                            ""
                        }
                    );
                }

                _ => println!("{:1$}{2:?}", "", SHIFT * depth, b),
            }
        }
    }

    if x.descriptors.len() > 0 {
        println!("");
        println!("Detailed timing descriptors:");
        for dt in &x.descriptors {
            println!(
                "{:1$}Resolution: {2}x{3}",
                "",
                SHIFT * depth,
                dt.horizontal_active_pixels,
                dt.vertical_active_lines
            );
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

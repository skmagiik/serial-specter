use clap::{builder::PossibleValue, Parser, Subcommand, ValueEnum};
use inline_colorization::*;
use serialport::SerialPort;
use std::{str::FromStr, time::{Duration, Instant}};


#[derive(Clone, Debug, Subcommand)]
enum PrintFormat{
    ASCII,
    HexDump,
    XXD
}

impl FromStr for PrintFormat {

    type Err = ();

    fn from_str(input: &str) -> Result<PrintFormat, Self::Err> {
        let filtered_string = input.to_uppercase();
        match filtered_string.as_str() {
            "ASCII"  => Ok(PrintFormat::ASCII),
            "HEXDUMP"  => Ok(PrintFormat::HexDump),
            "XXD" => Ok(PrintFormat::XXD),
            _      => Err(()),
        }
    }
}

impl ValueEnum for PrintFormat {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::ASCII, Self::HexDump, Self::XXD]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Self::ASCII => PossibleValue::new("ascii"),
            Self::HexDump => PossibleValue::new("hexdump"),
            Self::XXD => PossibleValue::new("xxd"),
        })
    }

}
#[derive(Parser)]
struct Cli{
    #[arg(short='d', long, help = "The first serial device that will be relayed")]
    device_primary: String,
    #[arg(short='D', long, help = "The second serial device that will be relayed")]
    device_secondary: String,
    #[arg(short='b', long, help = "The second serial device that will be relayed")]
    baudrate: u32,
    #[arg(long, help="Enables debug output messages", required=false)]
    debug_enabled: bool,
    #[arg(short='f', long, help = "Sets the console output format [ASCII, HexDump]", required=false, value_enum, default_value_t = PrintFormat::ASCII)]
    print_format: PrintFormat
}

fn print_ascii_char(c: char){
    if c.is_ascii_control() && !c.is_ascii_whitespace() {
        print!("[{:02X}]", c as u8);
    }
    else {
        print!("{}", c);
    }
}

fn print_hexdump(offset: usize, bytes: &Vec<u8>) {
    for (_i, chunk) in bytes.chunks(16).enumerate() {
        print!("{:08x}: ", offset);

        for byte in chunk {
            print!("{:02x} ", byte);
        }

        for _ in 0..(16 - chunk.len()) {
            print!("   ");
        }

        println!();
    }
}

fn print_xxd(offset: usize, bytes: &Vec<u8>) {
    for (_i, chunk) in bytes.chunks(16).enumerate() {
        print!("{:08x}: ", offset);

        for byte in chunk {
            print!("{:02x} ", byte);
        }

        for _ in 0..(16 - chunk.len()) {
            print!("   ");
        }

        print!("  ");

        for &byte in chunk {
            match byte {
                0x20..=0x7e => print!("{}", byte as char),
                _ => print!("."),
            }
        }

        println!();
    }
}

fn read_serial_data(device: &mut Box<dyn SerialPort>, secondary_device: &mut Box<dyn SerialPort>, output_buffer: &mut Vec<u8>) -> usize{
    let mut serial_buf: Vec<u8> = vec![0; 256];
    let bytes_read = device.read(serial_buf.as_mut_slice()).unwrap_or_default();
    if bytes_read > 0 {
        // println!("read {} bytes", bytes_read);
        serial_buf[0..bytes_read].into_iter().for_each( |x| {
            output_buffer.push(*x);
            // print!("{}", *x as char);
        });
        // println!("");
        secondary_device.write_all(&serial_buf[0..bytes_read]).expect("Failed to send to secondary device");
    }
    bytes_read
}

fn print_serial_data(data_buffer: &Vec<u8>, offset: &mut usize, print_format: &PrintFormat, print_color: &str){
    match print_format {
        PrintFormat::ASCII => {
            print!("{}", print_color);
            for a in data_buffer.iter() {
                print_ascii_char(*a as char);
            }
            println!("{color_reset}");
        },
        PrintFormat::HexDump => {
            print!("{}", print_color);
            let mut cur_line: Vec<u8> = vec![0; 0];
            for a in data_buffer.iter() {
                cur_line.push(*a);
                if cur_line.len() == 0x10 {
                    print_hexdump(*offset, &cur_line);
                    *offset += 0x10;
                    for _ in 0..0x10{
                        cur_line.remove(0);
                    }
                }
            }
            if cur_line.len() > 0 {
                print_hexdump(*offset, &cur_line);
                *offset += cur_line.len();
            }
            cur_line.clear();
            println!("{color_reset}");
        },
        PrintFormat::XXD => {
            print!("{}", print_color);
            let mut cur_line: Vec<u8> = vec![0; 0];
            for a in data_buffer.iter() {
                cur_line.push(*a);
                if cur_line.len() == 0x10 {
                    print_xxd(*offset, &cur_line);
                    *offset += 0x10;
                    for _ in 0..0x10{
                        cur_line.remove(0);
                    }
                }
            }
            if cur_line.len() > 0 {
                print_xxd(*offset, &cur_line);
                *offset += cur_line.len();
            }
            cur_line.clear();
            println!("{color_reset}");
        },
    }
}


fn main() {
    let args = Cli::parse();
    println!("Relaying between {:?} and {:?} @ {:?}bps", args.device_primary, args.device_secondary, args.baudrate);
    println!("Format: {:?}", args.print_format);

    let mut primary_device = serialport::new(args.device_primary, args.baudrate)
        .timeout(Duration::from_millis(1000))
        .open()
        .expect("Failed to open primary port");

    let mut secondary_device = serialport::new(args.device_secondary, args.baudrate)
        .timeout(Duration::from_millis(1000))
        .open()
        .expect("Failed to open secondary port");

    let _ = primary_device.write_data_terminal_ready(true);
    let _ = secondary_device.write_data_terminal_ready(true);

    let _ = primary_device.set_baud_rate(args.baudrate);
    let _ = secondary_device.set_baud_rate(args.baudrate);

    let _ = primary_device.set_data_bits(serialport::DataBits::Eight);
    let _ = secondary_device.set_data_bits(serialport::DataBits::Eight);

    let _ = primary_device.set_parity(serialport::Parity::None);
    let _ = secondary_device.set_parity(serialport::Parity::None);

    let mut primary_data_buffer: Vec<u8> = vec![0; 0];
    let mut secondary_data_buffer: Vec<u8> = vec![0; 0];

    let mut primary_time_read = Instant::now();
    let mut secondary_time_read = Instant::now();

    let mut primary_offset: usize = 0;
    let mut secondary_offset: usize = 0;

    loop {
        read_serial_data(&mut primary_device,&mut secondary_device, &mut primary_data_buffer);
        read_serial_data(&mut secondary_device,&mut primary_device, &mut secondary_data_buffer);

        if primary_data_buffer.contains(&0x0D) || primary_data_buffer.contains(&0x0A) || (primary_data_buffer.len() > 0 && primary_time_read.elapsed() > Duration::from_millis(75)){

            print_serial_data(&primary_data_buffer, &mut primary_offset, &args.print_format, color_green);

            primary_data_buffer.clear();
            primary_time_read = Instant::now();
        }
        if secondary_data_buffer.contains(&0x0D) || secondary_data_buffer.contains(&0x0A) || (secondary_data_buffer.len() > 0 && secondary_time_read.elapsed() > Duration::from_millis(75)){

            print_serial_data(&secondary_data_buffer, &mut secondary_offset, &args.print_format, color_blue);

            secondary_data_buffer.clear();
            secondary_time_read = Instant::now();
        }
    }
}

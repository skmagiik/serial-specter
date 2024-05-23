use clap::Parser;
use inline_colorization::*;
use serialport::SerialPort;
use std::time::{Duration, Instant};

#[derive(Parser)]
struct Cli{
    #[arg(short='d', long, help = "The first serial device that will be relayed")]
    device_primary: String,
    #[arg(short='D', long, help = "The second serial device that will be relayed")]
    device_secondary: String,
    #[arg(short='b', long, help = "The second serial device that will be relayed")]
    baudrate: u32,
}

fn print_ascii_char(c: char){
    if c.is_ascii_control() && !c.is_ascii_whitespace() {
        print!("[{:02X}]", c as u8);
    }
    else {
        print!("{}", c);
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

fn main() {
    let args = Cli::parse();
    println!("Relaying between {:?} and {:?} @ {:?}bps", args.device_primary, args.device_secondary, args.baudrate);

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

    
    loop {
        read_serial_data(&mut primary_device,&mut secondary_device, &mut primary_data_buffer);
        read_serial_data(&mut secondary_device,&mut primary_device, &mut secondary_data_buffer);

        if primary_data_buffer.contains(&0x0D) || primary_data_buffer.contains(&0x0A) || (primary_data_buffer.len() > 0 && primary_time_read.elapsed() > Duration::from_millis(75)){
            print!("{color_green}");
            for a in primary_data_buffer.iter() {
                print_ascii_char(*a as char);
            }
            println!("{color_reset}");
            primary_data_buffer.clear();
            primary_time_read = Instant::now();
        }
        if secondary_data_buffer.contains(&0x0D) || secondary_data_buffer.contains(&0x0A) || (secondary_data_buffer.len() > 0 && secondary_time_read.elapsed() > Duration::from_millis(75)){
            print!("{color_blue}");
            for a in secondary_data_buffer.iter() {
                print_ascii_char(*a as char);
            }
            println!("{color_reset}");
            secondary_data_buffer.clear();
            secondary_time_read = Instant::now();
        }


    }
}

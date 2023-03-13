use csvwriter::VeloPoint;
use pcap_parser::*;
use pcap_parser::traits::PcapReaderIterator;
use std::fs::File;
use std::f32::consts::PI;
use std::path::Path;
use std::process::exit;
use std::env;

use crate::csvwriter::CsvWriter;

mod csvwriter;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("please specify the target pcap");
        exit(-1);
    }
    let input = &args[1];
    let stem = Path::new(input).file_stem().unwrap();

    //let start = Instant::now();
    let file = File::open(input).unwrap();
    let mut num_packets = 0;
    let mut reader = LegacyPcapReader::new(65536, file).expect("LegacyPcapReader");

    let dir = format!("{}/", stem.to_str().unwrap());
    let mut csv_writer = CsvWriter::create(&dir, stem.to_str().unwrap());

    loop {
        match reader.next() {
            Ok((offset, block)) => {
                num_packets += 1;
                match block {
                    PcapBlockOwned::Legacy(packet) => {
                        // println!("{}", packet.data.len());
                        // etherのヘッダ長は14byte
                        let ether_data = &packet.data[14..];
                        // ipv4のヘッダ長は可変(基本20byte)
                        let ip_header_size = ((ether_data[0] & 15) * 4) as usize;
                        let packet_size = (((ether_data[2] as u32) << 8) + ether_data[3] as u32) as usize;
                        let ip_data = &ether_data[ip_header_size..packet_size];
                        // udpのヘッダ長は8byte
                        let udp_data = &ip_data[8..ip_data.len()];
                        parse_packet_body(udp_data, &mut csv_writer);
                    },
                    _ => ()
                }
                reader.consume(offset);
            },
            Err(PcapError::Eof) => break,
            Err(PcapError::Incomplete) => {
                reader.refill().unwrap();
            },
            Err(e) => panic!("error while reading: {:?}", e),
        }
    }

    println!("num_packets: {}", num_packets);
    //let end = start.elapsed();
    //println!("{}.{:03}sec", end.as_secs(), end.subsec_millis() / 1000)
}

fn parse_packet_body(packet_body: &[u8], writer: &mut CsvWriter) {
    // let pre_header = &packet_body[0..6];
    let header = &packet_body[6..12];
    let block_num = header[1] as u32;

    let body = &packet_body[12..1052];
    
    let tail = &packet_body[1052..1076];
    let return_mode = tail[10];
    let date_time = &tail[13..19];
    let timestamp = ((tail[22] as u32) << 24) 
                        + ((tail[21] as u32) << 16) 
                        + ((tail[20] as u32) << 8) 
                        + ((tail[19] as u32));

    for block_index in 0..block_num {
        let block_timestamp = calc_block_timestamp(date_time, timestamp, block_index+1, return_mode);
        let block_start = (block_index*130) as usize;
        parse_block(&body[block_start..block_start+130], block_timestamp, writer);
    }
}

fn calc_block_timestamp(date_time: &[u8], timestamp: u32, block_id: u32, return_mode: u8) -> u32 {
    let t0 = ((date_time[4] as u64) * 60 * 1000000 + (date_time[5] as u64) * 1000000 + timestamp as u64) as f32;
    if return_mode == 0x37 || return_mode == 0x38 {
        (t0 + 3.28 - 50.0 * (8.0 - block_id as f32)) as u32
    }
    else {
        (t0 + 3.28 - (50 * ((8 - block_id)/2)) as f32) as u32
    }
}

fn channel_to_v_angle(channel: i32) -> i32 {
    -channel + 16
}

fn calc_polar_coordinate(azimuth_deg: f32, v_angle_deg: f32, distance_m: f32) -> (f32,f32,f32) {
    let azimuth_rad = azimuth_deg * PI / 180.0;
    let v_angle_rad = v_angle_deg * PI / 180.0;
    let x = distance_m * v_angle_rad.cos() * azimuth_rad.sin();
    let y = distance_m * v_angle_rad.cos() * azimuth_rad.cos();
    let z = distance_m * v_angle_rad.sin();
    (x,y,z)
}

fn parse_block(packet_block: &[u8], block_timestamp: u32, writer: &mut CsvWriter) {
    let azimuth = ((packet_block[1] as u32) << 8) + (packet_block[0] as u32);
    for channel in 0..32 as u8 {
        let channel_timestamp = (block_timestamp as f32 + 1.512 * channel as f32 + 0.28) as u32;
        let v_angle = channel_to_v_angle(channel as i32);
        let channel_start = (2+channel*4) as usize;
        let channel_data = &packet_block[channel_start..channel_start+4];
        let distance = ((channel_data[1] as u32) << 8) + (channel_data[0] as u32);
        let reflectivity = channel_data[2];
        let (x,y,z) = calc_polar_coordinate(
            azimuth as f32 / 100.0, 
            v_angle as f32, 
            distance as f32 * 4.0 / 100.0);

        writer.write_row(VeloPoint { 
            reflectivity, 
            channel, 
            azimuth, 
            distance_m: distance as f32 * 4.0 / 100.0, 
            adjusted_time: channel_timestamp, 
            timestamp: channel_timestamp, 
            vertical_angle: v_angle as f32, 
            x, y, z })
    }
}

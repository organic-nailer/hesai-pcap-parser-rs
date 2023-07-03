use std::{fs::{File, OpenOptions, self}, path::Path};
use std::io::{BufWriter, Write};

use crate::{velopoint::VeloPoint, framewriter::FrameWriter};

pub struct CsvWriter<'v> {
    dir: &'v str,
    file_prefix: &'v str,
    current_filename: String,
    current_file: Option<BufWriter<File>>,
    previous_azimuth: u32,
    file_index: i32,
}

impl<'v> CsvWriter<'v> {
    pub fn create(dir: &'v str, file_prefix: &'v str) -> CsvWriter<'v> {
        fs::create_dir(dir).unwrap();
        CsvWriter { 
            dir, 
            file_prefix, 
            current_filename: "".to_string(), 
            current_file: None, 
            previous_azimuth: 0, 
            file_index: 0 
        }
    }

    fn init_file(&mut self) {
        match &self.current_file {
            Some(_) => (),
            None => {
                self.current_filename = format!("{0}{1}_{2:>04}.csv", self.dir, self.file_prefix, self.file_index);
                let path = Path::new(&self.current_filename);
                let mut new_file = BufWriter::with_capacity(262144, OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(path)
                    .unwrap());
                new_file.write(r#""intensity","laser_id","azimuth","distance_m","adjustedtime","timestamp","vertical_angle","Points_m_XYZ:0","Points_m_XYZ:1","Points_m_XYZ:2"
"#.as_bytes()).unwrap();
                self.current_file = Some(new_file);
                self.file_index += 1;
            }
        }
    }
}

impl<'v> FrameWriter for CsvWriter<'v> {
    fn write_row(&mut self, row: VeloPoint) {
        if self.current_file.is_none() || row.azimuth < self.previous_azimuth {
            // let old_file = self.current_file;
            self.current_file = None;
            self.init_file();
        }
        match self.current_file {
            Some(ref mut v) => {
                // row.to_string_w(v);
                v.write(row.to_string().as_bytes()).unwrap();
                self.previous_azimuth = row.azimuth;
            },
            None => ()
        }
    }

    fn finalize(&mut self) { }
}
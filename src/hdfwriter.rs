use std::{fs, path::Path};
use hdf5::Dataset;
use hdf5::File;
use ndarray::s;

use crate::framewriter::FrameWriter;
use crate::velopoint::VeloPoint;

pub struct HdfWriter {
    dir: String,
    file_prefix: String,
    previous_azimuth: u32,
    file_index: u32,
    buffer: Vec<VeloPoint>,
}

impl HdfWriter {
    pub fn create(dir: String, file_prefix: String) -> HdfWriter {
        fs::create_dir(dir.to_string()).unwrap();
        HdfWriter { 
            dir, 
            file_prefix,
            previous_azimuth: 0, 
            file_index: 0,
            buffer: Vec::new(),
        }
    }

    fn write_to_file(&mut self) {
        let current_filename = format!("{0}{1}_{2:>04}.h5", self.dir, self.file_prefix, self.file_index);
        let path = Path::new(&current_filename);
        let new_file = File::create(path).unwrap();
        self.file_index += 1;

        let group = new_file.create_group("frame").unwrap();
        
        let points_num = self.buffer.len();
        
        let dataset = group.new_dataset::<VeloPoint>()
            .shape([points_num])
            .create("points").unwrap();
        
        dataset.write(&self.buffer).unwrap();
    }
}

impl FrameWriter for HdfWriter {
    fn write_row(&mut self, row: VeloPoint) {
        if row.azimuth < self.previous_azimuth {
            if self.buffer.len() > 0 {
                self.write_to_file();
                self.buffer.clear();
                return;
            }
        }
        self.previous_azimuth = row.azimuth;
        self.buffer.push(row);
    }

    fn finalize(&mut self) {
        if self.buffer.len() > 0 {
            self.write_to_file();
            self.buffer.clear();
        }
    }
}
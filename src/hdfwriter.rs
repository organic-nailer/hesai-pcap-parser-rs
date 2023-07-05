use std::{path::Path};
use hdf5::File;

use crate::framewriter::FrameWriter;
use crate::velopoint::VeloPoint;

pub struct HdfWriter {
    previous_azimuth: u32,
    dataset_index: u32,
    file: File,
    buffer: Vec<VeloPoint>,
}

impl HdfWriter {
    pub fn create(filename: String) -> HdfWriter {
        let filename = format!("{}.h5", filename);
        let path = Path::new(&filename);
        let file = File::create(path).unwrap();
        HdfWriter {
            previous_azimuth: 0, 
            dataset_index: 0,
            file,
            buffer: Vec::new(),
        }
    }

    fn write_to_file(&mut self) {
        let points_num = self.buffer.len();
        
        let dataset_name = format!("frame{}", self.dataset_index);
        let dataset = self.file.new_dataset::<VeloPoint>()
            .shape([points_num])
            .create(&*dataset_name).unwrap();
        
        dataset.write(&self.buffer).unwrap();
        self.dataset_index += 1;
    }
}

impl FrameWriter for HdfWriter {
    fn write_row(&mut self, row: VeloPoint) {
        if row.azimuth < self.previous_azimuth {
            if self.buffer.len() > 0 {
                self.write_to_file();
                self.buffer.clear();
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
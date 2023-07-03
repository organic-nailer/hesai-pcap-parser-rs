use std::{fs, path::Path};
use hdf5::Dataset;
use hdf5::File;
use ndarray::s;

use crate::framewriter::FrameWriter;
use crate::velopoint::VeloPoint;

pub struct HdfWriter<'v> {
    dir: &'v str,
    file_prefix: &'v str,
    current_filename: String,
    current_dataset: Option<Dataset>,
    previous_azimuth: u32,
    file_index: u32,
    point_index: i32,
}

impl<'v> HdfWriter<'v> {
    pub fn create(dir: &'v str, file_prefix: &'v str) -> HdfWriter<'v> {
        fs::create_dir(dir).unwrap();
        HdfWriter { 
            dir, 
            file_prefix, 
            current_filename: "".to_string(), 
            current_dataset: None, 
            previous_azimuth: 0, 
            file_index: 0,
            point_index: 0,
        }
    }

    fn init_file(&mut self) {
        match &self.current_dataset {
            Some(_) => (),
            None => {
                self.current_filename = format!("{0}{1}_{2:>04}.h5", self.dir, self.file_prefix, self.file_index);
                let path = Path::new(&self.current_filename);
                let new_file = File::create(path).unwrap();
                self.file_index += 1;

                let group = new_file.create_group("frame").unwrap();
                let dataset = group.new_dataset::<VeloPoint>()
                    .shape([128000])
                    .create("points").unwrap();
                
                self.current_dataset = Some(dataset);
                self.point_index = 0;
            }
        }
    }
}

impl<'v> FrameWriter for HdfWriter<'v> {
    fn write_row(&mut self, row: VeloPoint) {
        if self.current_dataset.is_none() || row.azimuth < self.previous_azimuth {
            // let old_file = self.current_file;
            self.current_dataset = None;
            self.init_file();
        }
        match self.current_dataset {
            Some(ref mut dataset) => {
                self.previous_azimuth = row.azimuth;
                // dataset.resize(dataset.size() + 1).unwrap();
                dataset.write_slice(&[row], s![self.point_index..=self.point_index]).unwrap();
                self.point_index += 1;
            },
            None => ()
        }
    }
}
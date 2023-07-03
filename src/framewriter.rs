use crate::velopoint::VeloPoint;

pub trait FrameWriter {
    fn write_row(&mut self, row: VeloPoint);
}
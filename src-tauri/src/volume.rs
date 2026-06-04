use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeProperties {
    pub dimensions: (u32, u32, u32),
    pub spacing: (f64, f64, f64),
    pub origin: (f64, f64, f64),
    pub voxel_min: f64,
    pub voxel_max: f64,
}

pub struct VolumeRenderer {
    properties: VolumeProperties,
    transfer_function: TransferFunction,
}

pub struct TransferFunction {
    points: Vec<(f64, f64, [u8; 4])>,
}

impl TransferFunction {
    pub fn new() -> Self {
        TransferFunction {
            points: vec![
                (-1000.0, 0.0, [0, 0, 0, 0]),
                (-500.0, 0.2, [100, 100, 100, 50]),
                (0.0, 0.5, [200, 150, 100, 150]),
                (100.0, 0.8, [255, 200, 150, 200]),
                (500.0, 1.0, [255, 255, 255, 255]),
            ],
        }
    }

    pub fn sample(&self, value: f64) -> [u8; 4] {
        if self.points.len() < 2 {
            return [0, 0, 0, 0];
        }

        for i in 0..self.points.len() - 1 {
            let (v1, _, c1) = self.points[i];
            let (v2, _, c2) = self.points[i + 1];
            
            if value >= v1 && value <= v2 {
                let t = (value - v1) / (v2 - v1);
                return [
                    (c1[0] as f64 + t * (c2[0] as f64 - c1[0] as f64)) as u8,
                    (c1[1] as f64 + t * (c2[1] as f64 - c1[1] as f64)) as u8,
                    (c1[2] as f64 + t * (c2[2] as f64 - c1[2] as f64)) as u8,
                    (c1[3] as f64 + t * (c2[3] as f64 - c1[3] as f64)) as u8,
                ];
            }
        }

        if value < self.points[0].0 {
            self.points[0].2
        } else {
            self.points.last().unwrap().2
        }
    }
}

impl VolumeRenderer {
    pub fn new() -> Self {
        VolumeRenderer {
            properties: VolumeProperties {
                dimensions: (0, 0, 0),
                spacing: (1.0, 1.0, 1.0),
                origin: (0.0, 0.0, 0.0),
                voxel_min: 0.0,
                voxel_max: 0.0,
            },
            transfer_function: TransferFunction::new(),
        }
    }

    pub fn set_properties(&mut self, props: VolumeProperties) {
        self.properties = props;
    }

    pub fn render_slice(
        &self,
        volume: &[u16],
        dimensions: (u32, u32, u32),
        plane: &str,
        slice_index: usize,
        window_center: f64,
        window_width: f64,
    ) -> Vec<u8> {
        let (width, height, depth) = dimensions;
        
        match plane {
            "axial" => self.render_axial(volume, width, height, depth, slice_index, window_center, window_width),
            "sagittal" => self.render_sagittal(volume, width, height, depth, slice_index, window_center, window_width),
            "coronal" => self.render_coronal(volume, width, height, depth, slice_index, window_center, window_width),
            _ => Vec::new(),
        }
    }

    fn render_axial(
        &self,
        volume: &[u16],
        width: u32,
        height: u32,
        _depth: u32,
        slice_index: usize,
        window_center: f64,
        window_width: f64,
    ) -> Vec<u8> {
        let mut output = vec![0u8; (width * height * 4) as usize];
        let start = slice_index * width as usize * height as usize;
        
        for y in 0..height as usize {
            for x in 0..width as usize {
                let voxel_idx = start + y * width as usize + x;
                let value = if voxel_idx < volume.len() {
                    volume[voxel_idx] as f64
                } else {
                    0.0
                };
                let pixel = self.apply_window(value, window_center, window_width);
                let out_idx = (y * width as usize + x) * 4;
                output[out_idx] = pixel;
                output[out_idx + 1] = pixel;
                output[out_idx + 2] = pixel;
                output[out_idx + 3] = 255;
            }
        }
        
        output
    }

    fn render_sagittal(
        &self,
        volume: &[u16],
        width: u32,
        height: u32,
        depth: u32,
        slice_index: usize,
        window_center: f64,
        window_width: f64,
    ) -> Vec<u8> {
        let mut output = vec![0u8; (height * depth * 4) as usize];
        
        for z in 0..depth as usize {
            for y in 0..height as usize {
                let voxel_idx = z * width as usize * height as usize + y * width as usize + slice_index;
                let value = if voxel_idx < volume.len() {
                    volume[voxel_idx] as f64
                } else {
                    0.0
                };
                let pixel = self.apply_window(value, window_center, window_width);
                let out_idx = (z * height as usize + y) * 4;
                output[out_idx] = pixel;
                output[out_idx + 1] = pixel;
                output[out_idx + 2] = pixel;
                output[out_idx + 3] = 255;
            }
        }
        
        output
    }

    fn render_coronal(
        &self,
        volume: &[u16],
        width: u32,
        height: u32,
        depth: u32,
        slice_index: usize,
        window_center: f64,
        window_width: f64,
    ) -> Vec<u8> {
        let mut output = vec![0u8; (width * depth * 4) as usize];
        
        for z in 0..depth as usize {
            for x in 0..width as usize {
                let voxel_idx = z * width as usize * height as usize + slice_index * width as usize + x;
                let value = if voxel_idx < volume.len() {
                    volume[voxel_idx] as f64
                } else {
                    0.0
                };
                let pixel = self.apply_window(value, window_center, window_width);
                let out_idx = (z * width as usize + x) * 4;
                output[out_idx] = pixel;
                output[out_idx + 1] = pixel;
                output[out_idx + 2] = pixel;
                output[out_idx + 3] = 255;
            }
        }
        
        output
    }

    fn apply_window(&self, value: f64, window_center: f64, window_width: f64) -> u8 {
        let min = window_center - window_width / 2.0;
        let max = window_center + window_width / 2.0;
        
        if value <= min {
            0
        } else if value >= max {
            255
        } else {
            ((value - min) / window_width * 255.0) as u8
        }
    }
}

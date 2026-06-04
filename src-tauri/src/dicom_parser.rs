use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use dicom::object::open_file;
use dicom::pixeldata::PixelDecoder;
use walkdir::WalkDir;
use rayon::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomImage {
    pub width: u32,
    pub height: u32,
    pub pixel_data: Vec<u16>,
    pub window_center: f64,
    pub window_width: f64,
    pub slope: f64,
    pub intercept: f64,
    pub spacing: (f64, f64),
    pub slice_thickness: f64,
    pub patient_name: String,
    pub patient_id: String,
    pub study_date: String,
    pub series_description: String,
    pub modality: String,
    pub instance_number: u32,
    pub is_multiframe: bool,
    pub number_of_frames: u32,
    pub frame_time: f64,
    pub recommended_frame_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiframeData {
    pub width: u32,
    pub height: u32,
    pub number_of_frames: u32,
    pub frames: Vec<Vec<u16>>,
    pub frame_times: Vec<f64>,
    pub frame_time: f64,
    pub recommended_frame_rate: f64,
    pub window_center: f64,
    pub window_width: f64,
    pub slope: f64,
    pub intercept: f64,
    pub spacing: (f64, f64),
    pub patient_name: String,
    pub patient_id: String,
    pub study_date: String,
    pub series_description: String,
    pub modality: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiframeInfo {
    pub number_of_frames: u32,
    pub frame_time: f64,
    pub recommended_frame_rate: f64,
    pub actual_frame_rate: f64,
    pub has_frame_time_vector: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesInfo {
    pub series_uid: String,
    pub description: String,
    pub modality: String,
    pub num_slices: usize,
    pub patient_name: String,
    pub study_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeInfo {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub spacing_x: f64,
    pub spacing_y: f64,
    pub spacing_z: f64,
    pub window_center: f64,
    pub window_width: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MprSlice {
    pub width: u32,
    pub height: u32,
    pub pixel_data: Vec<u16>,
    pub plane: String,
    pub slice_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeData {
    pub dimensions: [u32; 3],
    pub spacing: [f64; 3],
    pub voxel_data: Vec<u16>,
}

pub struct DicomParser {
    current_image: Option<DicomImage>,
    volume_slices: Vec<DicomImage>,
    volume_data: Option<Vec<u16>>,
    volume_dimensions: (u32, u32, u32),
    volume_spacing: (f64, f64, f64),
    multiframe_data: Option<MultiframeData>,
}

impl Default for DicomParser {
    fn default() -> Self {
        Self::new()
    }
}

impl DicomParser {
    pub fn new() -> Self {
        DicomParser {
            current_image: None,
            volume_slices: Vec::new(),
            volume_data: None,
            volume_dimensions: (0, 0, 0),
            volume_spacing: (1.0, 1.0, 1.0),
            multiframe_data: None,
        }
    }

    pub fn load_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let obj = open_file(path)?;
        let image = self.extract_image_data(&obj)?;
        
        if image.is_multiframe {
            let multiframe = self.extract_multiframe_data(&obj, &image)?;
            self.multiframe_data = Some(multiframe);
        } else {
            self.multiframe_data = None;
        }
        
        self.current_image = Some(image);
        Ok(())
    }

    pub fn load_directory(&mut self, directory: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut dicom_files: Vec<String> = Vec::new();
        
        for entry in WalkDir::new(directory)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            if let Some(ext) = entry.path().extension() {
                if ext == "dcm" || ext == "dicom" || ext == "" {
                    dicom_files.push(entry.path().to_string_lossy().to_string());
                }
            }
        }

        let mut slices: Vec<DicomImage> = dicom_files
            .par_iter()
            .filter_map(|file| {
                open_file(file).ok().and_then(|obj| self.extract_image_data(&obj).ok())
            })
            .collect();

        slices.sort_by_key(|s| s.instance_number);
        
        self.volume_slices = slices;
        self.build_volume();
        
        if let Some(first) = self.volume_slices.first() {
            self.current_image = Some(first.clone());
        }

        Ok(())
    }

    fn extract_image_data(&self, obj: &dicom::object::DefaultDicomObject) -> Result<DicomImage, Box<dyn std::error::Error>> {
        let pixel_data = obj.decode_pixel_data()?;
        let raw_pixels = pixel_data.to_vec::<u16>()?;

        let number_of_frames = obj.element_by_name("NumberOfFrames")
            .ok()
            .and_then(|e| e.string().ok()?.parse::<u32>().ok())
            .unwrap_or(1);

        let is_multiframe = number_of_frames > 1;

        let frame_time = obj.element_by_name("FrameTime")
            .ok()
            .and_then(|e| e.float64().ok())
            .unwrap_or(0.0);

        let recommended_frame_rate = obj.element_by_name("RecommendedDisplayFrameRate")
            .ok()
            .and_then(|e| e.float64().ok())
            .unwrap_or(0.0);

        let window_center = obj.element_by_name("WindowCenter")
            .ok()
            .and_then(|e| e.float64().ok())
            .unwrap_or(40.0);
        
        let window_width = obj.element_by_name("WindowWidth")
            .ok()
            .and_then(|e| e.float64().ok())
            .unwrap_or(400.0);

        let slope = obj.element_by_name("RescaleSlope")
            .ok()
            .and_then(|e| e.float64().ok())
            .unwrap_or(1.0);
        
        let intercept = obj.element_by_name("RescaleIntercept")
            .ok()
            .and_then(|e| e.float64().ok())
            .unwrap_or(0.0);

        let pixel_spacing = obj.element_by_name("PixelSpacing")
            .ok()
            .and_then(|e| {
                let val = e.string().ok()?;
                let parts: Vec<&str> = val.split('\\').collect();
                if parts.len() >= 2 {
                    let x = parts[0].parse::<f64>().ok()?;
                    let y = parts[1].parse::<f64>().ok()?;
                    Some((x, y))
                } else {
                    None
                }
            })
            .unwrap_or((1.0, 1.0));

        let slice_thickness = obj.element_by_name("SliceThickness")
            .ok()
            .and_then(|e| e.float64().ok())
            .unwrap_or(1.0);

        let patient_name = obj.element_by_name("PatientName")
            .ok()
            .and_then(|e| e.string().ok())
            .unwrap_or_default()
            .to_string();

        let patient_id = obj.element_by_name("PatientID")
            .ok()
            .and_then(|e| e.string().ok())
            .unwrap_or_default()
            .to_string();

        let study_date = obj.element_by_name("StudyDate")
            .ok()
            .and_then(|e| e.string().ok())
            .unwrap_or_default()
            .to_string();

        let series_description = obj.element_by_name("SeriesDescription")
            .ok()
            .and_then(|e| e.string().ok())
            .unwrap_or_default()
            .to_string();

        let modality = obj.element_by_name("Modality")
            .ok()
            .and_then(|e| e.string().ok())
            .unwrap_or_default()
            .to_string();

        let instance_number = obj.element_by_name("InstanceNumber")
            .ok()
            .and_then(|e| e.string().ok()?.parse::<u32>().ok())
            .unwrap_or(0);

        let rows = obj.element_by_name("Rows")
            .ok()
            .and_then(|e| e.uint16().ok())
            .unwrap_or(0) as u32;

        let cols = obj.element_by_name("Columns")
            .ok()
            .and_then(|e| e.uint16().ok())
            .unwrap_or(0) as u32;

        let mut first_frame_pixels = raw_pixels.clone();
        if is_multiframe {
            let frame_size = (cols * rows) as usize;
            if first_frame_pixels.len() >= frame_size {
                first_frame_pixels = first_frame_pixels[..frame_size].to_vec();
            }
        }

        Ok(DicomImage {
            width: cols,
            height: rows,
            pixel_data: first_frame_pixels,
            window_center,
            window_width,
            slope,
            intercept,
            spacing: pixel_spacing,
            slice_thickness,
            patient_name,
            patient_id,
            study_date,
            series_description,
            modality,
            instance_number,
            is_multiframe,
            number_of_frames,
            frame_time,
            recommended_frame_rate,
        })
    }

    fn extract_multiframe_data(
        &self,
        obj: &dicom::object::DefaultDicomObject,
        image: &DicomImage,
    ) -> Result<MultiframeData, Box<dyn std::error::Error>> {
        let pixel_data = obj.decode_pixel_data()?;
        let raw_pixels = pixel_data.to_vec::<u16>()?;

        let width = image.width;
        let height = image.height;
        let num_frames = image.number_of_frames;
        let frame_size = (width * height) as usize;

        let mut frames = Vec::with_capacity(num_frames as usize);
        for i in 0..num_frames as usize {
            let start = i * frame_size;
            let end = start + frame_size;
            if end <= raw_pixels.len() {
                frames.push(raw_pixels[start..end].to_vec());
            } else {
                frames.push(vec![0u16; frame_size]);
            }
        }

        let mut frame_times = Vec::new();
        if let Ok(ft_vec) = obj.element_by_name("FrameTimeVector") {
            if let Ok(ft_str) = ft_vec.string() {
                let parts: Vec<&str> = ft_str.split('\\').collect();
                for part in parts {
                    if let Ok(t) = part.parse::<f64>() {
                        frame_times.push(t);
                    }
                }
            }
        }

        if frame_times.is_empty() && image.frame_time > 0.0 {
            for _ in 0..num_frames {
                frame_times.push(image.frame_time);
            }
        }

        if frame_times.is_empty() && image.recommended_frame_rate > 0.0 {
            let ft = 1000.0 / image.recommended_frame_rate;
            for _ in 0..num_frames {
                frame_times.push(ft);
            }
        }

        if frame_times.is_empty() {
            for _ in 0..num_frames {
                frame_times.push(33.33);
            }
        }

        while frame_times.len() < num_frames as usize {
            let last = *frame_times.last().unwrap_or(&33.33);
            frame_times.push(last);
        }

        let actual_frame_rate = if !frame_times.is_empty() {
            let avg_time: f64 = frame_times.iter().sum::<f64>() / frame_times.len() as f64;
            if avg_time > 0.0 {
                1000.0 / avg_time
            } else {
                0.0
            }
        } else {
            0.0
        };

        Ok(MultiframeData {
            width,
            height,
            number_of_frames: num_frames,
            frames,
            frame_times: frame_times.clone(),
            frame_time: if image.frame_time > 0.0 {
                image.frame_time
            } else if !frame_times.is_empty() {
                frame_times.iter().sum::<f64>() / frame_times.len() as f64
            } else {
                33.33
            },
            recommended_frame_rate: image.recommended_frame_rate,
            window_center: image.window_center,
            window_width: image.window_width,
            slope: image.slope,
            intercept: image.intercept,
            spacing: image.spacing,
            patient_name: image.patient_name.clone(),
            patient_id: image.patient_id.clone(),
            study_date: image.study_date.clone(),
            series_description: image.series_description.clone(),
            modality: image.modality.clone(),
        })
    }

    fn build_volume(&mut self) {
        if self.volume_slices.is_empty() {
            return;
        }

        let first = &self.volume_slices[0];
        let width = first.width as usize;
        let height = first.height as usize;
        let depth = self.volume_slices.len();

        let mut volume = vec![0u16; width * height * depth];

        for (z, slice) in self.volume_slices.iter().enumerate() {
            let slice_data = &slice.pixel_data;
            let start = z * width * height;
            let end = start + width * height;
            if end <= volume.len() && slice_data.len() == width * height {
                volume[start..end].copy_from_slice(slice_data);
            }
        }

        self.volume_data = Some(volume);
        self.volume_dimensions = (width as u32, height as u32, depth as u32);
        self.volume_spacing = (first.spacing.0, first.spacing.1, first.slice_thickness);
    }

    pub fn get_current_image(&self) -> DicomImage {
        self.current_image.clone().unwrap_or_else(|| DicomImage {
            width: 0,
            height: 0,
            pixel_data: Vec::new(),
            window_center: 40.0,
            window_width: 400.0,
            slope: 1.0,
            intercept: 0.0,
            spacing: (1.0, 1.0),
            slice_thickness: 1.0,
            patient_name: String::new(),
            patient_id: String::new(),
            study_date: String::new(),
            series_description: String::new(),
            modality: String::new(),
            instance_number: 0,
            is_multiframe: false,
            number_of_frames: 1,
            frame_time: 0.0,
            recommended_frame_rate: 0.0,
        })
    }

    pub fn get_multiframe_info(&self) -> Option<MultiframeInfo> {
        self.multiframe_data.as_ref().map(|m| {
            let actual_frame_rate = if !m.frame_times.is_empty() {
                let avg_time: f64 = m.frame_times.iter().sum::<f64>() / m.frame_times.len() as f64;
                if avg_time > 0.0 {
                    1000.0 / avg_time
                } else {
                    0.0
                }
            } else {
                0.0
            };

            MultiframeInfo {
                number_of_frames: m.number_of_frames,
                frame_time: m.frame_time,
                recommended_frame_rate: m.recommended_frame_rate,
                actual_frame_rate,
                has_frame_time_vector: m.frame_times.len() > 0 && 
                    m.frame_times.iter().any(|&t| t != m.frame_times[0]),
            }
        })
    }

    pub fn get_multiframe_data(&self) -> Option<MultiframeData> {
        self.multiframe_data.clone()
    }

    pub fn get_frame(&self, frame_index: usize) -> Option<Vec<u16>> {
        self.multiframe_data.as_ref().and_then(|m| {
            if frame_index < m.frames.len() {
                Some(m.frames[frame_index].clone())
            } else {
                None
            }
        })
    }

    pub fn get_frame_time(&self, frame_index: usize) -> f64 {
        self.multiframe_data.as_ref().and_then(|m| {
            if frame_index < m.frame_times.len() {
                Some(m.frame_times[frame_index])
            } else {
                None
            }
        }).unwrap_or(33.33)
    }

    pub fn get_series_info(&self) -> Vec<SeriesInfo> {
        let mut series_map: HashMap<String, Vec<&DicomImage>> = HashMap::new();
        
        for slice in &self.volume_slices {
            series_map.entry(slice.series_description.clone())
                .or_default()
                .push(slice);
        }

        series_map.into_iter().map(|(desc, slices)| {
            let first = slices.first().unwrap();
            SeriesInfo {
                series_uid: first.patient_id.clone(),
                description: desc,
                modality: first.modality.clone(),
                num_slices: slices.len(),
                patient_name: first.patient_name.clone(),
                study_date: first.study_date.clone(),
            }
        }).collect()
    }

    pub fn get_volume_info(&self) -> VolumeInfo {
        VolumeInfo {
            width: self.volume_dimensions.0,
            height: self.volume_dimensions.1,
            depth: self.volume_dimensions.2,
            spacing_x: self.volume_spacing.0,
            spacing_y: self.volume_spacing.1,
            spacing_z: self.volume_spacing.2,
            window_center: self.current_image.as_ref().map(|i| i.window_center).unwrap_or(40.0),
            window_width: self.current_image.as_ref().map(|i| i.window_width).unwrap_or(400.0),
        }
    }

    pub fn get_mpr_slice(&self, plane: &str, slice_index: usize) -> Result<MprSlice, Box<dyn std::error::Error>> {
        let volume = self.volume_data.as_ref()
            .ok_or("Volume not loaded")?;
        
        let (width, height, depth) = self.volume_dimensions;
        
        match plane {
            "axial" => {
                if slice_index >= depth as usize {
                    return Err("Slice index out of bounds".into());
                }
                let start = slice_index * width as usize * height as usize;
                let end = start + width as usize * height as usize;
                Ok(MprSlice {
                    width,
                    height,
                    pixel_data: volume[start..end].to_vec(),
                    plane: plane.to_string(),
                    slice_index,
                })
            }
            "sagittal" => {
                if slice_index >= width as usize {
                    return Err("Slice index out of bounds".into());
                }
                let mut slice_data = Vec::with_capacity(height as usize * depth as usize);
                for z in 0..depth as usize {
                    for y in 0..height as usize {
                        let idx = z * width as usize * height as usize + y * width as usize + slice_index;
                        slice_data.push(volume[idx]);
                    }
                }
                Ok(MprSlice {
                    width: height,
                    height: depth,
                    pixel_data: slice_data,
                    plane: plane.to_string(),
                    slice_index,
                })
            }
            "coronal" => {
                if slice_index >= height as usize {
                    return Err("Slice index out of bounds".into());
                }
                let mut slice_data = Vec::with_capacity(width as usize * depth as usize);
                for z in 0..depth as usize {
                    for x in 0..width as usize {
                        let idx = z * width as usize * height as usize + slice_index * width as usize + x;
                        slice_data.push(volume[idx]);
                    }
                }
                Ok(MprSlice {
                    width,
                    height: depth,
                    pixel_data: slice_data,
                    plane: plane.to_string(),
                    slice_index,
                })
            }
            _ => Err("Invalid plane type".into())
        }
    }

    pub fn get_volume_data(&self) -> Result<VolumeData, Box<dyn std::error::Error>> {
        let volume = self.volume_data.as_ref()
            .ok_or("Volume not loaded")?;
        
        Ok(VolumeData {
            dimensions: [self.volume_dimensions.0, self.volume_dimensions.1, self.volume_dimensions.2],
            spacing: [self.volume_spacing.0, self.volume_spacing.1, self.volume_spacing.2],
            voxel_data: volume.clone(),
        })
    }
}

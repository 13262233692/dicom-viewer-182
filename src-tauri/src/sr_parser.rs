use serde::{Serialize, Deserialize};
use dicom::object::open_file;
use dicom::object::Tag;
use dicom::object::InMemDicomObject;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SRDocument {
    pub patient_name: String,
    pub patient_id: String,
    pub study_date: String,
    pub study_description: String,
    pub series_description: String,
    pub modality: String,
    pub sop_class_uid: String,
    pub sop_instance_uid: String,
    pub completion_flag: String,
    pub verification_flag: String,
    pub content_date: String,
    pub content_time: String,
    pub root_node: SRNode,
    pub measurements: Vec<SRMeasurement>,
    pub conclusions: Vec<SRConclusion>,
    pub image_references: Vec<SRImageRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SRNode {
    pub node_id: String,
    pub value_type: String,
    pub relationship_type: String,
    pub concept_name: String,
    pub concept_code: String,
    pub text_value: String,
    pub numeric_value: Option<f64>,
    pub numeric_unit: String,
    pub children: Vec<SRNode>,
    pub image_reference: Option<SRImageRef>,
    pub spatial_coordinates: Option<SRSpatialCoord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SRMeasurement {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub concept_code: String,
    pub image_reference: Option<SRImageRef>,
    pub spatial_coordinates: Option<SRSpatialCoord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SRConclusion {
    pub text: String,
    pub code_value: String,
    pub code_meaning: String,
    pub image_reference: Option<SRImageRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SRImageRef {
    pub sop_class_uid: String,
    pub sop_instance_uid: String,
    pub frame_number: Option<u32>,
    pub referenced_frame_numbers: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SRSpatialCoord {
    pub graphic_type: String,
    pub graphic_data: Vec<(f64, f64)>,
    pub frame_of_reference_uid: String,
    pub referenced_image: Option<SRImageRef>,
}

pub struct SRParser {
    current_sr: Option<SRDocument>,
    node_counter: usize,
}

impl Default for SRParser {
    fn default() -> Self {
        Self::new()
    }
}

impl SRParser {
    pub fn new() -> Self {
        SRParser {
            current_sr: None,
            node_counter: 0,
        }
    }

    pub fn load_sr_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let obj = open_file(path)?;

        let modality = obj.element_by_name("Modality")
            .ok()
            .and_then(|e| e.string().ok())
            .unwrap_or_default()
            .to_string();

        if modality != "SR" {
            return Err("Not a DICOM SR file".into());
        }

        self.node_counter = 0;

        let root_node = {
            let inner: &InMemDicomObject = &obj;
            self.parse_content_item(inner)?
        };

        let mut measurements = Vec::new();
        let mut conclusions = Vec::new();
        let mut image_references = Vec::new();

        self.extract_measurements(&root_node, &mut measurements);
        self.extract_conclusions(&root_node, &mut conclusions);
        self.extract_image_references(&root_node, &mut image_references);

        let patient_name = get_string(&obj, "PatientName");
        let patient_id = get_string(&obj, "PatientID");
        let study_date = get_string(&obj, "StudyDate");
        let study_description = get_string(&obj, "StudyDescription");
        let series_description = get_string(&obj, "SeriesDescription");
        let sop_class_uid = get_string(&obj, "SOPClassUID");
        let sop_instance_uid = get_string(&obj, "SOPInstanceUID");
        let completion_flag = get_string(&obj, "CompletionFlag");
        let verification_flag = get_string(&obj, "VerificationFlag");
        let content_date = get_string(&obj, "ContentDate");
        let content_time = get_string(&obj, "ContentTime");

        self.current_sr = Some(SRDocument {
            patient_name,
            patient_id,
            study_date,
            study_description,
            series_description,
            modality,
            sop_class_uid,
            sop_instance_uid,
            completion_flag,
            verification_flag,
            content_date,
            content_time,
            root_node,
            measurements,
            conclusions,
            image_references,
        });

        Ok(())
    }

    fn next_node_id(&mut self) -> String {
        self.node_counter += 1;
        format!("node_{}", self.node_counter)
    }

    fn parse_content_item(
        &mut self,
        obj: &InMemDicomObject,
    ) -> Result<SRNode, Box<dyn std::error::Error>> {
        let value_type = get_string(obj, "ValueType");
        let relationship_type = get_string(obj, "RelationshipType");
        let (concept_name, concept_code) = self.extract_concept_name(obj);
        let text_value = self.extract_text_value(obj);
        let (numeric_value, numeric_unit) = self.extract_numeric_value(obj);
        let image_reference = self.extract_image_ref_from_obj(obj);
        let spatial_coordinates = self.extract_spatial_coordinates(obj);

        let mut children = Vec::new();
        if let Ok(content_seq) = obj.element(Tag(0x0040, 0xA730)) {
            if let Some(items) = content_seq.items() {
                for item in items {
                    if let Ok(child) = self.parse_content_item(item) {
                        children.push(child);
                    }
                }
            }
        }

        Ok(SRNode {
            node_id: self.next_node_id(),
            value_type,
            relationship_type,
            concept_name,
            concept_code,
            text_value,
            numeric_value,
            numeric_unit,
            children,
            image_reference,
            spatial_coordinates,
        })
    }

    fn extract_concept_name(
        &self,
        obj: &InMemDicomObject,
    ) -> (String, String) {
        let mut name = String::new();
        let mut code = String::new();

        if let Ok(cn) = obj.element(Tag(0x0040, 0xA043)) {
            if let Some(items) = cn.items() {
                for item in items {
                    let m = get_string(item, "CodeMeaning");
                    if !m.is_empty() { name = m; }
                    let c = get_string(item, "CodeValue");
                    if !c.is_empty() { code = c; }
                }
            }
        }

        (name, code)
    }

    fn extract_text_value(&self, obj: &InMemDicomObject) -> String {
        obj.element(Tag(0x0040, 0xA160))
            .ok()
            .and_then(|e| e.string().ok())
            .unwrap_or_default()
            .to_string()
    }

    fn extract_numeric_value(
        &self,
        obj: &InMemDicomObject,
    ) -> (Option<f64>, String) {
        let mut value: Option<f64> = None;
        let mut unit = String::new();

        if let Ok(mv_seq) = obj.element(Tag(0x0040, 0xA300)) {
            if let Some(items) = mv_seq.items() {
                for item in items {
                    if let Ok(nv) = item.element(Tag(0x0040, 0xA30A)) {
                        if let Ok(val) = nv.float64() {
                            value = Some(val);
                        } else if let Ok(val) = nv.string() {
                            value = val.parse::<f64>().ok();
                        }
                    }

                    if let Ok(mu_seq) = item.element(Tag(0x0040, 0x08EA)) {
                        if let Some(items2) = mu_seq.items() {
                            for item2 in items2 {
                                let m = get_string(item2, "CodeMeaning");
                                if !m.is_empty() { unit = m; }
                            }
                        }
                    }
                }
            }
        }

        (value, unit)
    }

    fn extract_image_ref_from_obj(
        &self,
        obj: &InMemDicomObject,
    ) -> Option<SRImageRef> {
        if let Ok(ref_seq) = obj.element(Tag(0x0008, 0x1199)) {
            if let Some(items) = ref_seq.items() {
                for item in items {
                    if let Some(img_ref) = parse_image_ref_item(item) {
                        return Some(img_ref);
                    }
                }
            }
        }

        if let Ok(img_seq) = obj.element(Tag(0x0040, 0xA132)) {
            if let Some(items) = img_seq.items() {
                for item in items {
                    if let Some(img_ref) = parse_image_ref_item(item) {
                        return Some(img_ref);
                    }
                }
            }
        }

        None
    }

    fn extract_spatial_coordinates(
        &self,
        obj: &InMemDicomObject,
    ) -> Option<SRSpatialCoord> {
        if let Ok(scoord_seq) = obj.element(Tag(0x0070, 0x0001)) {
            if let Some(items) = scoord_seq.items() {
                for item in items {
                    if let Some(sc) = parse_scoord_item(item) {
                        return Some(sc);
                    }
                }
            }
        }

        if let Ok(scoord_seq) = obj.element(Tag(0x0040, 0xA130)) {
            if let Some(items) = scoord_seq.items() {
                for item in items {
                    if let Some(sc) = parse_scoord_item(item) {
                        return Some(sc);
                    }
                }
            }
        }

        None
    }

    fn extract_measurements(&self, node: &SRNode, measurements: &mut Vec<SRMeasurement>) {
        if node.value_type == "NUM" {
            if let Some(value) = node.numeric_value {
                let mut img_ref = node.image_reference.clone();
                let mut spatial = node.spatial_coordinates.clone();

                for child in &node.children {
                    if child.value_type == "SCOORD" || child.value_type == "IMAGE" {
                        if spatial.is_none() {
                            spatial = child.spatial_coordinates.clone();
                        }
                        if img_ref.is_none() {
                            img_ref = child.image_reference.clone();
                        }
                    }
                }

                measurements.push(SRMeasurement {
                    name: if node.concept_name.is_empty() {
                        "Unknown Measurement".to_string()
                    } else {
                        node.concept_name.clone()
                    },
                    value,
                    unit: node.numeric_unit.clone(),
                    concept_code: node.concept_code.clone(),
                    image_reference: img_ref,
                    spatial_coordinates: spatial,
                });
            }
        }

        for child in &node.children {
            self.extract_measurements(child, measurements);
        }
    }

    fn extract_conclusions(&self, node: &SRNode, conclusions: &mut Vec<SRConclusion>) {
        let is_conclusion_container = node.value_type == "CONTAINER"
            && (node.concept_name.contains("Conclusion")
                || node.concept_name.contains("Finding")
                || node.concept_name.contains("Impression")
                || node.concept_name.contains("Diagnosis")
                || node.concept_name.contains("Assessment"));

        if is_conclusion_container {
            for child in &node.children {
                if child.value_type == "TEXT" && !child.text_value.is_empty() {
                    conclusions.push(SRConclusion {
                        text: child.text_value.clone(),
                        code_value: child.concept_code.clone(),
                        code_meaning: child.concept_name.clone(),
                        image_reference: child.image_reference.clone(),
                    });
                } else if child.value_type == "CODE" {
                    conclusions.push(SRConclusion {
                        text: if child.text_value.is_empty() {
                            child.concept_name.clone()
                        } else {
                            child.text_value.clone()
                        },
                        code_value: child.concept_code.clone(),
                        code_meaning: child.concept_name.clone(),
                        image_reference: child.image_reference.clone(),
                    });
                }
            }
        }

        if node.value_type == "TEXT"
            && !node.text_value.is_empty()
            && (node.concept_name.contains("Conclusion")
                || node.concept_name.contains("Finding")
                || node.concept_name.contains("Impression"))
        {
            conclusions.push(SRConclusion {
                text: node.text_value.clone(),
                code_value: node.concept_code.clone(),
                code_meaning: node.concept_name.clone(),
                image_reference: node.image_reference.clone(),
            });
        }

        if node.value_type == "CODE"
            && (node.concept_name.contains("Conclusion")
                || node.concept_name.contains("Finding"))
        {
            conclusions.push(SRConclusion {
                text: if node.text_value.is_empty() {
                    node.concept_name.clone()
                } else {
                    node.text_value.clone()
                },
                code_value: node.concept_code.clone(),
                code_meaning: node.concept_name.clone(),
                image_reference: node.image_reference.clone(),
            });
        }

        for child in &node.children {
            self.extract_conclusions(child, conclusions);
        }
    }

    fn extract_image_references(&self, node: &SRNode, refs: &mut Vec<SRImageRef>) {
        if let Some(img_ref) = &node.image_reference {
            if !refs.iter().any(|r| r.sop_instance_uid == img_ref.sop_instance_uid) {
                refs.push(img_ref.clone());
            }
        }

        if let Some(scoord) = &node.spatial_coordinates {
            if let Some(img_ref) = &scoord.referenced_image {
                if !refs.iter().any(|r| r.sop_instance_uid == img_ref.sop_instance_uid) {
                    refs.push(img_ref.clone());
                }
            }
        }

        for child in &node.children {
            self.extract_image_references(child, refs);
        }
    }

    pub fn get_sr_document(&self) -> Option<SRDocument> {
        self.current_sr.clone()
    }

    pub fn get_sr_measurements(&self) -> Vec<SRMeasurement> {
        self.current_sr
            .as_ref()
            .map(|sr| sr.measurements.clone())
            .unwrap_or_default()
    }

    pub fn get_sr_conclusions(&self) -> Vec<SRConclusion> {
        self.current_sr
            .as_ref()
            .map(|sr| sr.conclusions.clone())
            .unwrap_or_default()
    }

    pub fn get_sr_image_references(&self) -> Vec<SRImageRef> {
        self.current_sr
            .as_ref()
            .map(|sr| sr.image_references.clone())
            .unwrap_or_default()
    }

    pub fn get_measurements_for_image(&self, sop_instance_uid: &str) -> Vec<SRMeasurement> {
        self.current_sr
            .as_ref()
            .map(|sr| {
                sr.measurements
                    .iter()
                    .filter(|m| {
                        m.image_reference
                            .as_ref()
                            .map(|r| r.sop_instance_uid == sop_instance_uid)
                            .unwrap_or(false)
                            || m.spatial_coordinates
                            .as_ref()
                            .and_then(|sc| sc.referenced_image.as_ref())
                            .map(|r| r.sop_instance_uid == sop_instance_uid)
                            .unwrap_or(false)
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
}

fn get_string(obj: &InMemDicomObject, name: &str) -> String {
    obj.element_by_name(name)
        .ok()
        .and_then(|e| e.string().ok())
        .unwrap_or_default()
        .to_string()
}

fn parse_image_ref_item(item: &InMemDicomObject) -> Option<SRImageRef> {
    let sop_class = get_string(item, "ReferencedSOPClassUID");
    let sop_instance = get_string(item, "ReferencedSOPInstanceUID");

    if sop_instance.is_empty() {
        return None;
    }

    let frame_number = item.element_by_name("ReferencedFrameNumber")
        .ok()
        .and_then(|e| e.string().ok()?.parse::<u32>().ok());

    let mut ref_frames = Vec::new();
    if let Ok(rf) = item.element_by_name("ReferencedFrameNumber") {
        if let Ok(val) = rf.string() {
            for part in val.split('\\') {
                if let Ok(n) = part.parse::<u32>() {
                    ref_frames.push(n);
                }
            }
        }
    }

    Some(SRImageRef {
        sop_class_uid: sop_class,
        sop_instance_uid: sop_instance,
        frame_number,
        referenced_frame_numbers: ref_frames,
    })
}

fn parse_scoord_item(item: &InMemDicomObject) -> Option<SRSpatialCoord> {
    let graphic_type = item.element(Tag(0x0070, 0x0023))
        .ok()
        .and_then(|e| e.string().ok())
        .unwrap_or_default()
        .to_string();

    if graphic_type.is_empty() {
        return None;
    }

    let mut graphic_data = Vec::new();
    if let Ok(gd) = item.element(Tag(0x0070, 0x0022)) {
        if let Ok(val) = gd.string() {
            let coords: Vec<&str> = val.split('\\').collect();
            let mut i = 0;
            while i + 1 < coords.len() {
                if let (Ok(x), Ok(y)) = (coords[i].parse::<f64>(), coords[i + 1].parse::<f64>()) {
                    graphic_data.push((x, y));
                }
                i += 2;
            }
        }
    }

    let frame_of_ref = get_string(item, "FrameOfReferenceUID");

    let ref_image = parse_image_ref_item(item);

    Some(SRSpatialCoord {
        graphic_type,
        graphic_data,
        frame_of_reference_uid: frame_of_ref,
        referenced_image: ref_image,
    })
}


use std::fs::File;

use spex::parsing::XmlReader;

use crate::types::DdsPaths;
use crate::types::OtherValues;
use serde_json::{Value, Map};


pub fn extract_dds_paths(mdl_file: std::path::PathBuf) -> DdsPaths {
    let mut dds_paths = crate::types::DdsPaths::default();
    let mdl_file = std::fs::File::open(mdl_file).unwrap();

    let xml_doc = XmlReader::parse_auto(mdl_file).unwrap();
    let inputs: Vec<_> = xml_doc.root().all("input").iter().collect();

    for input in inputs {
        let name = input.req("semantic").text().unwrap();
        let value = input.req("value").text().unwrap();
                
        let value = if !value.starts_with("/") && !value.starts_with("\\") {
            format!("/{}.dds", value)
        } else {
            format!("{}.dds", value.replace("\\", "/"))
        };

        match name {
            #[allow(unused_assignments)]
            "PaletteMap" => dds_paths.palette_map = value.clone(),
            #[allow(unused_assignments)]
            "PaletteMaskMap" => dds_paths.palette_mask_map = value.clone(),
            #[allow(unused_assignments)]
            "DiffuseMap" => dds_paths.diffuse_map = value.clone(),
            #[allow(unused_assignments)]
            "GlossMap" => dds_paths.gloss_map = value.clone(),
            #[allow(unused_assignments)]
            "RotationMap1" => dds_paths.rotation_map = value.clone(),
            _ => {}
        }
    }

    return dds_paths;
}

pub fn extract_palletes(mdl_file: std::path::PathBuf) -> OtherValues {
    let mut other_values = OtherValues::default();
    let mdl_file = std::fs::File::open(mdl_file).unwrap();

    let xml_doc = XmlReader::parse_auto(mdl_file).unwrap();
    let inputs: Vec<_> = xml_doc.root().all("input").iter().collect();

    for input in inputs {
        let name = input.req("semantic").text().unwrap();
        let value = input.req("value").text().unwrap();
                
        let value_split = value.split(",").map(|s| s.to_string()).collect::<Vec<String>>();

        match name {
            #[allow(unused_assignments)]
            "palette1" => other_values.palette1 = value_split,
            #[allow(unused_assignments)]
            "palette2" => other_values.palette2 = value_split,
            #[allow(unused_assignments)]
            "palette1Specular" => other_values.palette1Specular = value_split,
            #[allow(unused_assignments)]
            "palette2Specular" => other_values.palette2Specular = value_split,
            #[allow(unused_assignments)]
            "palette1MetallicSpecular" => other_values.palette1MetallicSpecular = value_split,
            #[allow(unused_assignments)]
            "palette2MetallicSpecular" => other_values.palette2MetallicSpecular = value_split,
            _ => {}
        }
    }

    return other_values;
}

pub fn extract_garments(garments: Vec<String>) -> Map<String, Value> {
    let read_config = std::fs::read_to_string("config.toml").unwrap();
    let config: toml::Value = toml::from_str(&read_config).unwrap();
    let ressources_path_cfg = config["general"]["resources_path"].as_str().unwrap();

    let ressources_path = std::path::PathBuf::from(ressources_path_cfg);

    let mut garment_map_temp = Map::new();

    for (index, garment) in garments.iter().enumerate() {
        // how do i get the index here?
        let index = index + 1;

        let path = ressources_path.join(format!("art/dynamic/garmenthue/{}.xml", garment));
        let xml_file = File::open(path).unwrap();
        let xml_doc = XmlReader::parse_auto(xml_file).unwrap();
        let inputs = xml_doc.root();

        let hue = inputs.req("Hue").text().unwrap();
        let saturation = inputs.req("Saturation").text().unwrap();
        let brightness = inputs.req("Brightness").text().unwrap();
        let contrast = inputs.req("Contrast").text().unwrap();

        let metallicspecular: Vec<&str> = inputs.req("Metallicspecular").text().unwrap().split(",").take(3).collect();
        let specular: Vec<&str> = inputs.req("Specular").text().unwrap().split(",").take(3).collect();

        garment_map_temp.insert(format!("palette{}", index), Value::Array(vec![hue, saturation, brightness, contrast].into_iter().map(|x| Value::String(x.to_string())).collect()));
        garment_map_temp.insert(format!("palette{}MetallicSpecular", index), Value::Array(metallicspecular.into_iter().map(|x| Value::String(x.to_string())).collect()));
        garment_map_temp.insert(format!("palette{}Specular", index), Value::Array(specular.into_iter().map(|x| Value::String(x.to_string())).collect()));
    }

    return garment_map_temp;
}

pub fn extract_flush(mdl_file: std::path::PathBuf) -> Vec<f64> {
    let mdl_file = std::fs::File::open(mdl_file).unwrap();
    let xml_doc = XmlReader::parse_auto(mdl_file).unwrap();
    let inputs = xml_doc.root();

    let mut flush: Vec<f64> = Vec::new();

    for input in inputs.all("input").iter() {
        let name = input.req("semantic").text().unwrap();
        let value = input.req("value").text().unwrap();
                
        if name != "FlushTone" {
            continue;
        }
        
        flush = value.split(",").map(|s| s.parse().unwrap()).collect();
        break;
    }

    return flush;
}

pub fn extract_flesh(mdl_file: std::path::PathBuf) -> f64 {
    let mdl_file = std::fs::File::open(mdl_file).unwrap();
    let xml_doc = XmlReader::parse_auto(mdl_file).unwrap();
    let inputs = xml_doc.root();

    let mut flesh: f64 = 0.1;

    for input in inputs.all("input").iter() {
        let name = input.req("semantic").text().unwrap();
        let value = input.req("value").text().unwrap();
                
        if name != "FleshBrightness" {
            continue;
        }
        
        flesh = value.parse().unwrap();
        break;
    }

    return flesh;
}
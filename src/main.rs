use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::result::Result;
use kuchikiki::{self as kuchiki};
use kuchikiki::traits::TendrilSink;
use serde_json::{json, Value, Map};

mod types;
mod funcs;

use types::{DdsPaths, OtherValues};

#[tokio::main]
async fn main() -> Result<(),()> {
    let args: Vec<String> = env::args().collect();

    // create config.toml file if it doesn't exist
    if !Path::new("config.toml").exists() {
        let mut file = File::create("config.toml").unwrap();
        file.write_all(b"[general]\nresources_path = \"\"").unwrap();
        println!("config.toml file created. Please fill in the resources_path field.");

        return Ok(());
    }

    let read_config = std::fs::read_to_string("config.toml").unwrap();
    let config: toml::Value = toml::from_str(&read_config).unwrap(); // yes, i know i should have used json here because i already have the json crate but i like toml more.
    let ressources_path_cfg = config["general"]["resources_path"].as_str().unwrap();

    let ressources_path = std::path::PathBuf::from(ressources_path_cfg);

    let input_tmpo = if args.len() > 1 {
        args[1].clone()
    } else {
        println!("Please enter the URL of the NPC you want to convert:");
        let mut input_tmp = String::new();
        std::io::stdin().read_line(&mut input_tmp).unwrap();
        input_tmp.trim().to_string()
    };

    println!("Starting Converter....");

    let input = input_tmpo.clone();

    let binding = input.clone();
    let name_of_npc = binding.split("/").last().unwrap();
    let client = reqwest::Client::new();

    let response = client.get(input.clone()).send().await;
    let response = match response {
        Ok(response) => response,
        Err(_) => return Err(())
    };

    let body = response.text().await;
    let body = match body {
        Ok(body) => body,
        Err(_) => return Err(())
    };

    let document = kuchiki::parse_html().one(body);

    let body_type = document.document_node.select_first("table#npp0 tbody tr:nth-child(2) td span").unwrap();
    let body_type = body_type.text_contents();

    let items_list = document.document_node.select("table.nice tbody tr").unwrap();
    let mut data_test = Vec::new();

    for item in items_list {
        let textcontent = item.text_contents();
        if textcontent.contains("\u{200b}") { // TODO: Find a better fix!
            continue;
        }

        let slot_name = item.as_node().select_first("td:nth-child(1)").unwrap();
        let slot_name: Vec<String> = slot_name.text_contents().split("(").map(String::from).collect();
        let slot_name = slot_name[0].trim();
    
        let mut models: Vec<String> = Vec::new();
        let mut materials: Vec<String> = Vec::new();
        let mut garments = Vec::new();

        let garments_list = item.as_node().select("td:nth-child(2) div").unwrap();

        for garment_node in garments_list {
            let title = {
                let attributes = garment_node.attributes.borrow();
                attributes.get("title").unwrap_or_default().to_string()
            };
            if title.is_empty() {
                continue;
            }
            // title example: title="Secondary Hue: /art/dynamic/garmenthue/garmenthue_ss_h05_p.xml. and i want to extract the last part of the path
            let parts: String = title.split("/").last().unwrap().to_string();
            let garment = parts.split(".").next().unwrap();
            garments.push(garment.to_string());
        }
                
        // creature

        let model_name = item.as_node().select_first("td:nth-child(2) span");
        if model_name.is_err() {
            continue;
        }
        let model_name = model_name.unwrap();
        models.push(format!("art/dynamic/{}/model/{}.gr2", slot_name, model_name.text_contents()));

        let extra_models = item.as_node().select("td:nth-child(2) ul li span").unwrap();

        for model in extra_models {
            let attributes = model.attributes.borrow();
            let title = attributes.get("title").is_some();
            if !title {
                materials.push(format!("art/shaders/materials/{}.mat",model.text_contents()));
                continue;
            }

            let model_path = ressources_path.join(format!("art/dynamic/{}/model/{}.gr2", slot_name, model.text_contents()));

            if model_path.exists() {
                models.push(format!("art/dynamic/{}/model/{}.gr2", slot_name, model.text_contents()));
                continue;
            }
        }

        if materials.is_empty() {
            continue;
        }

        let normal_material = materials[0].clone();
        let normal_dds = funcs::extract_dds_paths(ressources_path.join(normal_material.clone()));
        let normal_flush = funcs::extract_flush(ressources_path.join(normal_material.clone()));
        let normal_flesh = funcs::extract_flesh(ressources_path.join(normal_material.clone()));

        let dds = normal_dds;

        let eye_dds = if materials.len() > 1 {
            let eye_material = materials[1].clone();
            funcs::extract_dds_paths(ressources_path.join(eye_material))
        } else {
            DdsPaths::default()
        };

        let eye_pallets = if materials.len() > 1 {
            funcs::extract_palletes(ressources_path.join(materials[1].clone()))
        } else {
            OtherValues::default()
        };

        let eye_flush: Vec<f64> = if materials.len() > 1 {
            funcs::extract_flush(ressources_path.join(materials[1].clone()))
        } else {
            vec![0.0,0.0,0.0,0.0]
        };

        let eye_flesh: f64 = if materials.len() > 1 {
            funcs::extract_flesh(ressources_path.join(materials[1].clone()))
        } else {
            0.1
        };

        #[allow(unused_variables)]
        let dds_eye = eye_dds;
        let eye_mat_other_values = eye_pallets;

        let mut garment_map_temp = funcs::extract_garments(garments.clone());

        if garment_map_temp.is_empty() {
            // get the default values from the .mat file
            let default_pallets = funcs::extract_palletes(ressources_path.join(materials[0].clone()));
            garment_map_temp.insert("palette1".to_string(), Value::Array(default_pallets.palette1.into_iter().map(|x| Value::String(x)).collect()));
            garment_map_temp.insert("palette2".to_string(), Value::Array(default_pallets.palette2.into_iter().map(|x| Value::String(x)).collect()));
            garment_map_temp.insert("palette1Specular".to_string(), Value::Array(default_pallets.palette1Specular.into_iter().map(|x| Value::String(x)).collect()));
            garment_map_temp.insert("palette2Specular".to_string(), Value::Array(default_pallets.palette2Specular.into_iter().map(|x| Value::String(x)).collect()));
            garment_map_temp.insert("palette1MetallicSpecular".to_string(), Value::Array(default_pallets.palette1MetallicSpecular.into_iter().map(|x| Value::String(x)).collect()));
            garment_map_temp.insert("palette2MetallicSpecular".to_string(), Value::Array(default_pallets.palette2MetallicSpecular.into_iter().map(|x| Value::String(x)).collect()));

            println!("No garmenthue found for {}. Using default values from the .mat file.", slot_name);
        }

        let mut slot = Map::new();

        slot.insert("slotName".to_string(), Value::String(slot_name.to_string()));
        slot.insert("models".to_string(), Value::Array(models.clone().into_iter().map(|x| Value::String(format!("/{}",x))).collect()));

        let mut material_info = Map::new();
        material_info.insert("matPath".to_string(), Value::String(format!("/{}",materials[0])));
        let mut dds_info = Map::new();
        dds_info.insert("paletteMap".to_string(), Value::String(dds.palette_map.to_string()));
        dds_info.insert("paletteMaskMap".to_string(), Value::String(dds.palette_mask_map.to_string()));
        dds_info.insert("diffuseMap".to_string(), Value::String(dds.diffuse_map.to_string()));
        dds_info.insert("glossMap".to_string(), Value::String(dds.gloss_map.to_string()));
        dds_info.insert("rotationMap".to_string(), Value::String(dds.rotation_map.to_string()));
        
        material_info.insert("ddsPaths".to_string(), Value::Object(dds_info));

        let mut othervalues = Map::new();
        othervalues.insert("derived".to_string(), Value::String("Garment".to_string()));
        othervalues.insert("flush".to_string(), Value::Array(normal_flush.into_iter().map(|x| Value::Number(serde_json::Number::from_f64(x).unwrap())).collect()));
        othervalues.insert("fleshBrightness".to_string(), Value::from(normal_flesh));

        let empty_array: Vec<String> = Vec::new();
        for key in ["palette1", "palette2", "palette1Specular", "palette2Specular", "palette1MetallicSpecular", "palette2MetallicSpecular"] {
            if !garment_map_temp.contains_key(key) {
                garment_map_temp.insert(
                    key.to_string(), 
                    Value::Array(empty_array.clone().into_iter().map(|x| Value::String(x.into())).collect())
                );
            }
        }

        for (key, value) in garment_map_temp.iter() {
            othervalues.insert(key.clone(), value.clone());
        }

        material_info.insert("otherValues".to_string(), Value::Object(othervalues));

        if slot_name == "creature" || slot_name == "head" {
            // eye_mat_other_values
            let mut eye_material_info = Map::new();

            let mut dds_info_eye = Map::new();
            dds_info_eye.insert("paletteMap".to_string(), Value::String(dds_eye.palette_map.to_string()));
            dds_info_eye.insert("paletteMaskMap".to_string(), Value::String(dds_eye.palette_mask_map.to_string()));
            dds_info_eye.insert("diffuseMap".to_string(), Value::String(dds_eye.diffuse_map.to_string()));
            dds_info_eye.insert("glossMap".to_string(), Value::String(dds_eye.gloss_map.to_string()));
            dds_info_eye.insert("rotationMap".to_string(), Value::String(dds_eye.rotation_map.to_string()));
            eye_material_info.insert("ddsPaths".to_string(), Value::Object(dds_info_eye));

            let mut othervalues_eye = Map::new();
            othervalues_eye.insert("derived".to_string(), Value::String("Eye".to_string()));
            othervalues_eye.insert("flush".to_string(), Value::Array(eye_flush.into_iter().map(|x| Value::Number(serde_json::Number::from_f64(x).unwrap())).collect()));
            othervalues_eye.insert("fleshBrightness".to_string(), Value::from(eye_flesh as f64));

            // eye_mat_other_values
            othervalues_eye.insert("palette1".to_string(), Value::Array(eye_mat_other_values.palette1.into_iter().map(|x| Value::String(x)).collect()));
            othervalues_eye.insert("palette1MetallicSpecular".to_string(), Value::Array(eye_mat_other_values.palette1MetallicSpecular.into_iter().map(|x| Value::String(x)).collect()));
            othervalues_eye.insert("palette1Specular".to_string(), Value::Array(eye_mat_other_values.palette1Specular.into_iter().map(|x| Value::String(x)).collect()));
            othervalues_eye.insert("palette2".to_string(), Value::Array(eye_mat_other_values.palette2.into_iter().map(|x| Value::String(x)).collect()));
            othervalues_eye.insert("palette2MetallicSpecular".to_string(), Value::Array(eye_mat_other_values.palette2MetallicSpecular.into_iter().map(|x| Value::String(x)).collect()));
            othervalues_eye.insert("palette2Specular".to_string(), Value::Array(eye_mat_other_values.palette2Specular.into_iter().map(|x| Value::String(x)).collect()));

            for key in ["palette1", "palette2", "palette1Specular", "palette2Specular", "palette1MetallicSpecular", "palette2MetallicSpecular"] {
                if !othervalues_eye.contains_key(key) {
                    othervalues_eye.insert(
                        key.to_string(), 
                        Value::Array(empty_array.clone().into_iter().map(|x| Value::String(x.into())).collect())
                    );
                }
            }

            eye_material_info.insert("otherValues".to_string(), Value::Object(othervalues_eye));
            material_info.insert("eyeMatInfo".to_string(), Value::Object(eye_material_info));
        }


        slot.insert("materialInfo".to_string(), Value::Object(material_info));
        
        data_test.push(slot);
    }

    let json_value = json!(data_test);

    // TODO: Add skincolor/haircolor support.

    let json_string = serde_json::to_string_pretty(&json_value).unwrap();

    // replace all \\ with / in the json_string
    let json_string = json_string.replace("\\\\", "/");
    // write to file in exe directory

    std::fs::create_dir_all(name_of_npc).unwrap();
    let mut file = File::create(format!("{}/paths.json",name_of_npc).as_str()).unwrap();
    file.write(json_string.as_bytes()).unwrap();

    // if body_type is only 3 characters long, add new at the end
    let skeleton_name = if body_type.len() == 3 {
        format!("{}new", body_type)
    } else {
        body_type.to_string()
    };

    let mut skeleton_file = File::create(format!("{}/skeleton.json",name_of_npc).as_str()).unwrap();
    skeleton_file.write(json!({"path": format!("/art/dynamic/spec/{}_skeleton.gr2", skeleton_name)}).to_string().as_bytes()).unwrap();

    println!("Files saved in the directory: {}", name_of_npc);
    println!("Thanks for using the converter! (made by @viewmatrix)");

    Ok(())
}

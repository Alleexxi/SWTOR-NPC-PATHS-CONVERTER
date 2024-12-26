
#[derive(Debug)]
pub struct DdsPaths {
    pub palette_map: String,
    pub palette_mask_map: String,
    pub diffuse_map: String,
    pub gloss_map: String,
    pub rotation_map: String,
}

impl Default for DdsPaths {
    fn default() -> Self {
        DdsPaths {
            palette_map: String::from("/art/defaultassets/black.dds"),
            palette_mask_map: String::from("/art/defaultassets/black.dds"),
            diffuse_map: String::from("/art/defaultassets/black.dds"),
            gloss_map: String::from("/art/defaultassets/black.dds"),
            rotation_map: String::from("/art/defaultassets/black.dds"),
        }
    }
}


#[derive(Debug, Default)]
pub struct EyeMatOtherValues {
    pub palette1: Vec<String>,
    pub palette2: Vec<String>,
    pub palette1Specular: Vec<String>,
    pub palette2Specular: Vec<String>,
    pub palette1MetallicSpecular: Vec<String>,
    pub palette2MetallicSpecular: Vec<String>,
}

#[derive(Debug, Default)]
pub struct OtherValues {
    pub palette1: Vec<String>,
    pub palette2: Vec<String>,
    pub palette1Specular: Vec<String>,
    pub palette2Specular: Vec<String>,
    pub palette1MetallicSpecular: Vec<String>,
    pub palette2MetallicSpecular: Vec<String>,
}
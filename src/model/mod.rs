use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AppDataStruct {
    pub series: Vec<Series>,
}

#[derive(Serialize, Deserialize)]
pub struct Series {
    pub name: String,
    pub products: Vec<Product>,
}

#[derive(Serialize, Deserialize)]
pub struct Product {
    pub name: String,
    pub firmware: Vec<Firmware>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Firmware {
    pub name: String,
    pub version: String,
    pub fw_path: String,
    pub chip_series: String,
    pub chip_type: String,
}

impl Default for Firmware {
    fn default() -> Self {
        Self {
            name: Default::default(),
            version: Default::default(),
            fw_path: Default::default(),
            chip_type: Default::default(),
            chip_series: Default::default(),
        }
    }
}

impl AppDataStruct {
    pub fn new() -> Self {
        Self { series: Vec::new() }
    }

    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::read_to_string(path)?;
        let app_data: AppDataStruct = serde_json::from_str(&data)?;
        Ok(app_data)
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    pub fn get_series_by_name(&self, name: &str) -> Option<&Series> {
        self.series.iter().find(|s| s.name == name)
    }

    pub fn get_product_by_name(&self, series_name: &str, product_name: &str) -> Option<&Product> {
        self.get_series_by_name(series_name)
            .and_then(|s| s.products.iter().find(|p| p.name == product_name))
    }

    pub fn get_firmware_by_name(
        &self,
        series_name: &str,
        product_name: &str,
        firmware_name: &str,
    ) -> Option<&Firmware> {
        self.get_product_by_name(series_name, product_name)
            .and_then(|p| p.firmware.iter().find(|f| f.name == firmware_name))
    }
}

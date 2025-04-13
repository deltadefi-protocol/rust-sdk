use whisky::WError;

use super::Api;

pub struct App {
    pub api: Api,
    pub path_url: String,
}

impl App {
    pub fn new(api: Api) -> Self {
        App {
            api,
            path_url: "/app".to_string(),
        }
    }

    pub async fn get_terms_and_conditions(&self) -> Result<String, WError> {
        let url = format!("{}/terms-and-conditions", self.path_url);
        self.api.get(&url).await
    }

    pub async fn get_hydra_cycle(&self) -> Result<String, WError> {
        let url = format!("{}/hydra-cycle", self.path_url);
        self.api.get(&url).await
    }
}

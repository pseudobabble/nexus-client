use std::env;
use std::env::VarError;
use std::path::Path;
use std::io::Cursor;

use reqwest::blocking::{Client};
use serde::{Serialize, Deserialize};
use serde_json::value::Value;


pub struct NexusClient {
    base_url: String,
    url_path: String,
    api_version: String,
    client: Client
}

impl Default for NexusClient {
    fn default() -> Self {
        let nexus_url: String = env::var("NEXUS_URL").unwrap();
        NexusClient {
            base_url: nexus_url,
            url_path: "/service/rest".to_string(),
            api_version: "/v1".to_string(),
            client: Client::new()
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
struct AssetHashes {
    sha1: Option<String>,
    sha256: Option<String>,
    sha512: Option<String>,
    md5: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Asset {
    downloadUrl: String,
    path: String,
    format: String,
    checksum: AssetHashes,
    contentType: String,
    lastModified: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchItem {
    id: String,
    repository: String,
    format: String,
    group: Option<String>,
    name: String,
    version: String,
    assets: Vec<Asset>,
    tags: Vec<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchReturnBody {
    items: Vec<SearchItem>,
    continuationToken: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Repository {
    name: String,
    format: String,
    url: String,
    attributes: Value
}



impl NexusClient {

    const QUERYSTRING_DELIMITER: &'static str = "?";
    const QUERYSTRING_JOINER: &'static str = "&";

    pub fn new() -> NexusClient {
        NexusClient {
            ..Default::default()
        }
    }

    pub fn with_url(base_url: &str) -> NexusClient {
        NexusClient {
            base_url: base_url.to_owned(),
            ..Default::default()
        }
    }

    pub fn construct(base_url: &str, url_path: &str, api_version: &str) -> NexusClient {
        NexusClient {
            base_url: base_url.to_string(),
            url_path: url_path.to_string(),
            api_version: api_version.to_string(),
            client: Client::new()
        }
    }

    fn http_search(&self, repository: &str, package_name: Option<&str>, version: Option<&str>, continuation_token: Option<String>) -> Result<SearchReturnBody, VarError>{

        let default_package_name = &"".to_string();  // this feels a bit silly
        let package_name: &str = package_name.unwrap_or(default_package_name);

        let default_version = &"".to_string();
        let version: &str = version.unwrap_or(default_version);

        let continuation_querystring: &str = &"".to_string();
        if continuation_token.is_some() {
            let continuation_querystring: &str = &[NexusClient::QUERYSTRING_JOINER, "continuationToken=", &continuation_token.unwrap()].concat();
        };

        let nexus_user: String = env::var("NEXUS_TOKEN_NAME")?;
        let nexus_password: String = env::var("NEXUS_TOKEN_SECRET")?;

        let request_url = "".to_string() +
            &self.base_url +
            &self.url_path +
            &self.api_version +
            &"/search".to_string() +
            NexusClient::QUERYSTRING_DELIMITER +
            &["repository=", repository, NexusClient::QUERYSTRING_JOINER].concat() +
            &["name=", package_name, NexusClient::QUERYSTRING_JOINER].concat() +
            &["version=", version].concat() +
            &continuation_querystring
        ;

        let response = self.client.get(&request_url).basic_auth(nexus_user, Some(nexus_password)).send();

        match response {
            Ok(response) => {
                match response.text() {
                    Ok(text) => {
                        let search_response: SearchReturnBody = serde_json::from_str(&text).unwrap();
                        Ok(search_response)
                    }
                    Err(err) => {
                        println!("Response content error: {:?}", err);
                        Err(VarError::NotPresent)
                    }
                }
            }
            Err(err) => {
                println!("Response was not OK: {:?}", err);
                Err(VarError::NotPresent)
            }
        }
    }

    pub fn list_repositories(self) -> Result<Vec<Repository>, VarError> {
        let nexus_user: String = env::var("NEXUS_TOKEN_NAME").unwrap();
        let nexus_password: String = env::var("NEXUS_TOKEN_SECRET").unwrap();

        let request_url = "".to_string() +
            &self.base_url +
            &self.url_path +
            &self.api_version +
            &"/repositories"
        ;
        let response = self.client.get(&request_url).basic_auth(nexus_user, Some(nexus_password)).send();

        match response {
            Ok(response) => {
                match response.text() {
                    Ok(text) => {
                        let search_response: Vec<Repository> = serde_json::from_str(&text).unwrap();
                        Ok(search_response)
                    }
                    Err(err) => {
                        println!("Response content error: {:?}", err);
                        Err(VarError::NotPresent)
                    }
                }
            }
            Err(err) => {
                println!("Response was not OK: {:?}", err);
                Err(VarError::NotPresent)
            }
        }
    }

    pub fn list_packages(&self, repository: &str) -> Result<Vec<String>, VarError> {
        let results: Vec<SearchItem> = self.search(repository, None, None).unwrap();
        let mut packages: Vec<String> = Vec::new();

        for result in results {
            packages.push(result.name);
        }

        Ok(packages)
    }

    pub fn search(&self, repository: &str, package_name: Option<&str>, version: Option<&str>) -> Result<Vec<SearchItem>, VarError> {
        let mut search_items: Vec<SearchItem> = Vec::new();
        let mut continuation_token: Option<String> = None;

        loop {
            let search_return = self.http_search(repository, package_name, version, continuation_token)?;
            search_items.append(&mut search_return.items.clone());
            continuation_token = match search_return.continuationToken {
                Some(token) => Some(token),
                None => None
            };

            if !continuation_token.is_some() {
                break;
            }
        }

        Ok(search_items)
    }

    pub fn upload() -> () {
        ();
    }

    pub fn download(&self, repository: &str, package_name: Option<&str>, version: Option<&str>) -> () {
        let nexus_user: String = env::var("NEXUS_TOKEN_NAME").unwrap();
        let nexus_password: String = env::var("NEXUS_TOKEN_SECRET").unwrap();
        let package_info = self.http_search(repository, package_name, version, None).ok();
        match package_info {
            Some(package_info) => {
                for item in package_info.items {
                    for asset in item.assets {
                        let filename = Path::new(&asset.path).file_name().unwrap();
                        let download_url = asset.downloadUrl.clone();
                        println!("\nDownloading {:#?}", filename.clone());

                        let response = self.client.get(&download_url).basic_auth(nexus_user.clone(), Some(nexus_password.clone())).send().expect("Client error");

                        // TODO: download to Downloads or specified location
                        let mut file = std::fs::File::create(filename).expect("Error creating file");
                        let mut content =  Cursor::new(response.bytes().expect("Error retrieving response content"));
                        std::io::copy(&mut content, &mut file).ok();
                    }
                }
            }

            None => {
                println!("No package was found for {:#?} {:#?} {:#?}", repository, package_name, version);
            }
        }

        ();
    }
}

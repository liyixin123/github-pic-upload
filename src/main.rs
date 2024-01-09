use base64::{engine::general_purpose, Engine};
use config::{Config, File, FileFormat};
use lazy_static::lazy_static;
use rand::Rng;
use reqwest::{
    header::{HeaderMap, HeaderValue, USER_AGENT},
    redirect::Policy,
};
use serde::{Deserialize, Serialize};
use std::{fs::read, sync::RwLock};

lazy_static! {
    static ref SETTINGS: RwLock<Config> = RwLock::new(
        Config::builder()
            .add_source(File::new("settings", FileFormat::Toml))
            .build()
            .unwrap()
    );
    static ref TOKEN: String = SETTINGS.read().unwrap().get::<String>("TOKEN").unwrap();
    static ref USER: String = SETTINGS.read().unwrap().get::<String>("USER").unwrap();
    static ref REPO: String = SETTINGS.read().unwrap().get::<String>("REPO").unwrap();
    static ref PATH: String = SETTINGS.read().unwrap().get::<String>("PATH").unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
struct Committer {
    name: String,
    email: String,
}

#[derive(Serialize)]
struct UploadRequest<'a> {
    message: &'static str,
    committer: Committer,
    content: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
struct Author {
    date: String,
    email: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Parent {
    html_url: String,
    sha: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Commit {
    author: Author,
    committer: Committer,
    html_url: String,
    message: String,
    node_id: String,
    parents: Vec<Parent>,
    sha: String,
    tree: Tree,
    url: String,
    verification: Verification,
}

#[derive(Debug, Serialize, Deserialize)]
struct Tree {
    sha: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Verification {
    payload: Option<String>,
    reason: String,
    signature: Option<String>,
    verified: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ContentLinks {
    git: String,
    html: String,
    #[serde(rename = "self")]
    self_link: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Content {
    _links: ContentLinks,
    download_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseFromGithub {
    commit: Commit,
    content: Content,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

fn generate_filename() -> String {
    let current_time = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
    let random_number: u32 = rand::thread_rng().gen_range(10000..99999);
    format!("{}_{}", current_time, random_number)
}

fn upload_file(file_data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let file_name = generate_filename() + ".png"; // 文件名
    let url = format!(
        "https://api.github.com/repos/{}/{}/contents/{}/{}",
        USER.as_str(),
        REPO.as_str(),
        PATH.as_str(),
        file_name
    );

    let content = file_base64(&file_data);

    let data = UploadRequest {
        message: "message",
        committer: Committer {
            name: USER.to_string(),
            email: "haha@liyixin.fun".to_string(),
        },
        content: &content,
    };

    let client = reqwest::blocking::Client::builder()
        .tcp_keepalive(None)
        // .tcp_keepalive(None)
        // .pool_max_idle_per_host(0)
        .redirect(Policy::limited(10))
        .timeout(None)
        .build()?;
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("reqwest"));
    let token = format!("token {}", TOKEN.as_str());
    headers.insert(
        reqwest::header::AUTHORIZATION,
        HeaderValue::from_str(token.as_str()).unwrap(),
    );
    headers.insert(
        "X-GitHub-Api-Version",
        HeaderValue::from_static("2022-11-28"),
    );
    let response = client.put(&url).headers(headers).json(&data).send()?;

    let json_value: ResponseFromGithub = response.json()?;
    let formatted_json = serde_json::to_string_pretty(&json_value)?;

    println!("{}", formatted_json);

    let markdown_link = format!(
        "![{}](https://cdn.jsdelivr.net/gh/{}/{}/{}/{})",
        file_name,
        USER.as_str(),
        REPO.as_str(),
        PATH.as_str(),
        file_name
    );

    println!("{}", markdown_link);

    let _ = clipboard_anywhere::set_clipboard(markdown_link.as_str());

    Ok(())
}

fn file_base64(data: &[u8]) -> String {
    general_purpose::STANDARD.encode(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Read;
    use tempfile::tempdir;

    #[test]
    fn test_upload_file() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Generate a temporary file name using the same logic as generate_filename
        let temp_file_name = generate_filename() + ".png";
        let temp_file_path = temp_dir.path().join(&temp_file_name);

        // Copy the test image file to the temporary file path
        fs::copy("./test/test_image1.png", &temp_file_path).expect("Failed to copy test image");

        // Read the content of the temporary file
        let mut file_content = Vec::new();
        File::open(&temp_file_path)
            .expect("Failed to open temporary file")
            .read_to_end(&mut file_content)
            .expect("Failed to read temporary file");

        // Call the upload_file function with the temporary file content
        let result = upload_file(file_content);

        // Assert that the result is Ok
        assert!(result.is_ok(), "Error: {:?}", result.err());

        // Optionally, you can add additional assertions or validations here

        // Print a message indicating the success of the test
        println!("Test successful!");

        // Cleanup: Delete the temporary directory and its contents
        temp_dir
            .close()
            .expect("Failed to close temporary directory");
    }
    #[test]
    fn test_serde_response() {
        let input_str = r#"
        {
            "commit": {
                "author": {
                    "date": "2024-01-08T11:50:31Z",
                    "email": "haha@liyixin.fun",
                    "name": "liyixin123"
                },
                "committer": {
                    "date": "2024-01-08T11:50:31Z",
                    "email": "haha@liyixin.fun",
                    "name": "liyixin123"
                },
                "html_url": "https://github.com/liyixin123/blog-img/commit/c93533f751486cf78b3df70a8f8dae7355f59428",
                "message": "message",
                "node_id": "C_kwDOKCzzftoAKGM5MzUzM2Y3NTE0ODZjZjc4YjNkZjcwYThmOGRhZTczNTVmNTk0Mjg",
                "parents": [
                    {
                        "html_url": "https://github.com/liyixin123/blog-img/commit/18c1d0320b096d7dc3aa1f8dc003dd44b5c923f4",
                        "sha": "18c1d0320b096d7dc3aa1f8dc003dd44b5c923f4",
                        "url": "https://api.github.com/repos/liyixin123/blog-img/git/commits/18c1d0320b096d7dc3aa1f8dc003dd44b5c923f4"
                    }
                ],
                "sha": "c93533f751486cf78b3df70a8f8dae7355f59428",
                "tree": {
                    "sha": "bdf915c4098c1d5035ecb580a2025ddc73532adb",
                    "url": "https://api.github.com/repos/liyixin123/blog-img/git/trees/bdf915c4098c1d5035ecb580a2025ddc73532adb"
                },
                "url": "https://api.github.com/repos/liyixin123/blog-img/git/commits/c93533f751486cf78b3df70a8f8dae7355f59428",
                "verification": {
                    "payload": null,
                    "reason": "unsigned",
                    "signature": null,
                    "verified": false
                }
            },
            "content": {
                "_links": {
                    "git": "https://api.github.com/repos/liyixin123/blog-img/git/blobs/cec2bca28af8d6b7fd16691c1dd6753bb7d6db98",
                    "html": "https://github.com/liyixin123/blog-img/blob/main/cli/20240108195024_79177.png",
                    "self": "https://api.github.com/repos/liyixin123/blog-img/contents/cli/20240108195024_79177.png?ref=main"
                },
                "download_url": "https://raw.githubusercontent.com/liyixin123/blog-img/main/cli/20240108195024_79177.png"
            }
        }
    "#;

        // 将 JSON 字符串解析为 serde_json::Value 类型
        let json_value: ResponseFromGithub =
            serde_json::from_str(input_str).expect("Failed to parse JSON");

        // 将 serde_json::Value 转换为格式化的 JSON 字符串
        let formatted_json =
            serde_json::to_string_pretty(&json_value).expect("Failed to format JSON");

        // 打印格式化后的 JSON
        println!("{}", formatted_json);
    }
}

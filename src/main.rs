use reqwest::{
    header::{ACCEPT, AUTHORIZATION},
    Result,
};
use serde::Deserialize;
use std::env;

#[derive(Deserialize, Debug)]
struct ApiResponse {
    total_count: u32,
    items: Vec<Repository>,
}

#[derive(Deserialize, Debug)]
struct Repository {
    full_name: String,
}

struct Repositories {
    search_str: String,
    repositories: Vec<Repository>,
    client: reqwest::Client,
    page: u32,
    per_page: u32,
    total: u32,
    token: String,
}

impl Repositories {
    async fn new(search_str: String, token: String) -> Result<Self>{

        Ok(Repositories {
            search_str,
            repositories: Vec::new(),
            client: reqwest::Client::new(),
            page: 0,
            per_page: 30,
            total: 0,
            token,
        })
    }

    async fn fetch_next_page(&mut self) -> Result<bool> {
        if self.page>0 &&self.page * self.per_page >=self.total{
            return Ok(false);
        }

        self.page+=1;

        let url = format!(
            "https://api.github.com/search/repositories?q={}&page={}&per_page={}",
            self.search_str, self.page, self.per_page
        );

        let token = format!("bearer {}", self.token);

        let response = self
            .client
            .get(&url)
            .header(ACCEPT, "application/vnd.github+json")
            .header(AUTHORIZATION, &token)
            .header("x-github-api-version", "2022-11-28")
            .header("user-agent", "rust")
            .send()
            .await?
            .json::<ApiResponse>()
            .await?;

        self.repositories.extend(response.items);
        self.total = response.total_count;

        Ok(true)
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut repo_full_names = Vec::new();
    let search_str = env::var("SEARCH_STR").unwrap();
    let gh_token = env::var("GH_TOKEN").unwrap();
    let file_path = env::var("FILE_PATH").unwrap();

    let mut repos = Repositories::new(search_str, gh_token).await?;

    while let Ok(has_more_repos) = repos.fetch_next_page().await {
       if !has_more_repos{
            break;
        }
    }

    for repo in &repos.repositories{
        repo_full_names.push(repo.full_name.clone());
    }

    std::fs::write(file_path, repo_full_names.join("\n")).unwrap();
    Ok(())
}


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
    repositories: <Vec<Repository> as IntoIterator>::IntoIter,
    client: reqwest::Client,
    page: u32,
    per_page: u32,
    total: u32,
    token: String,
}

impl Repositories {
    async fn new() -> Result<Self> {
        let search_str = env::var("SEARCH_STR").unwrap();
        let gh_token = env::var("GH_TOKEN").unwrap();

        Ok(Repositories {
            search_str: search_str.to_owned(),
            repositories: vec![].into_iter(),
            client: reqwest::Client::new(),
            page: 0,
            per_page: 30,
            total: 0,
            token: gh_token,
        })
    }

    async fn try_next(&mut self) -> Result<Option<Repository>> {
        if let Some(repo) = self.repositories.next() {
            return Ok(Some(repo));
        }

        if self.page > 0 && self.page * self.per_page >= self.total {
            return Ok(None);
        }

        self.page += 1;
        let url = format!(
            "https://api.github.com/search/repositories?q={}&page={}&per_page={}",
            self.search_str, self.page, self.per_page
        );

        let token = format!("bearer {}", self.token,);

        let response = self
            .client
            .get(url)
            .header(ACCEPT, "application/vnd.github+json")
            .header(AUTHORIZATION, &token)
            .header("x-github-api-version", "2022-11-28")
            .header("user-agent", "rust")
            .send()
            .await?
            .json::<ApiResponse>()
        .await?;
        self.repositories = response.items.into_iter();
        self.total = response.total_count;
        Ok(self.repositories.next())
    }
}

// impl Iterator for Repositories {
//     type Item = Result<Repository>;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         match self.try_next() {
//             Ok(Some(repo)) => Some(Ok(repo)),
//             Ok(None) => None,
//             Err(err) => Some(Err(err)),
//         }
//     }
// }

#[tokio::main]
async fn main() -> Result<()> {
    let mut repo_full_names = Vec::new();
    let file_path = env::var("FILE_PATH").unwrap();
    let mut repos = Repositories::new().await?;

    while let Some(repo) = repos.try_next().await? {
        
        // println!("reverse dependency: {}", repo?.full_name);
        repo_full_names.push(repo.full_name);
    }
    // for repo in Repositories::new().await? {
    //     // println!("reverse dependency: {}", repo?.full_name);
    //     repo_full_names.push(repo?.full_name);
    // }

    std::fs::write(file_path, repo_full_names.join("\n")).unwrap();
    Ok(())
}

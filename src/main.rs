use std::env;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Resp {
    devlogs: Vec<Devlog>
}

#[derive(Deserialize, Debug)]
struct Devlog {
    id: u32,
    body: String,
}

fn main() {
    let token = env::var("FT_API_KEY").expect("Failed to get API key from env");
    let url = "https://flavortown.hackclub.com/api/v1/devlogs";
    let client = reqwest::blocking::Client::new();
    let resp: Resp = client.get(url).bearer_auth(token)
        .send().expect("Failed to fetch")
        .json().expect("Failed to parse");
    
    for devlog in resp.devlogs.iter() {
        println!("{:?}", devlog);
    }
}

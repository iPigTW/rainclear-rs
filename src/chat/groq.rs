use std::env;

use dotenv::dotenv;
use serde_json::Value;

use super::History;

pub async fn get_response(history: Vec<History>) -> Result<String, String> {
    dotenv().ok();
    let client = reqwest::Client::new();
    let api_key = env::var("GROQ_API_KEY").unwrap();
    let payload = format!(
        r#"{{
        "model": "llama-3.3-70b-versatile",
        "messages": 
{}
        ,
        "temperature": 0.5,
        "max_tokens": 1024,
        "top_p": 1,
        "stop": null,
        "stream": false
}}
    "#,
        serde_json::to_string(&history).unwrap()
    );
    println!("{}", payload);
    let resp = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .body(payload)
        .send()
        .await;
    match resp {
        Ok(m) => Ok(
            m.json::<Value>().await.unwrap()["choices"][0]["message"]["content"]
                .as_str()
                .unwrap()
                .to_string(),
        ),
        Err(e) => Err(e.to_string()),
    }
}

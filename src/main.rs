use reqwest;

// tokio let's us use "async" on our main function
#[tokio::main]
async fn main() {
    // chaining .await will yield our query result
    let result = reqwest::get("https://api.spotify.com/v1/search").await;
    println!("{:?}", result);
}

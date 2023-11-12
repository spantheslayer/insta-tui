use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use select::{document::Document, predicate::{Class, Name}};
use reqwest::blocking::get;
use image::io::Reader as ImageReader;
use std::io::Cursor;
use viuer::Config;

struct InstagramUser {
    username: String,
}

struct Post {
    caption: String,
    photo_url: String,
}

fn load_usernames_from_file(file_path: &str) -> io::Result<Vec<InstagramUser>> {
    let mut users = Vec::new();

    let file = File::open(&Path::new(file_path))?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        users.push(InstagramUser { username: line });
    }

    Ok(users)
}

async fn fetch_posts_for_user(user: &InstagramUser) -> Result<Vec<Post>, reqwest::Error> {
    let url = format!("https://www.instagram.com/{}/", user.username);
    let resp = reqwest::get(&url).await?;
    let body = resp.text().await?;

    let document = Document::from(body.as_str());

    let mut posts = Vec::new();

    for node in document.find(Class("v1Nh3")) {
        let caption_node = node.find(Class("C4VMK")).next();
        let photo_node = node.find(Name("img")).next();

        match (caption_node, photo_node) {
            (Some(caption_node), Some(photo_node)) => {
                let caption = caption_node.text();
                let photo_url = photo_node.attr("src").unwrap_or_default().to_string();
                posts.push(Post { caption, photo_url });
            }
            _ => {
                eprintln!("Failed to parse post for user: {}", user.username);
            }
        }
    }

    Ok(posts)
}

fn display_posts_in_terminal(posts: Vec<Post>) -> Result<(), Box<dyn std::error::Error>> {
    for post in posts {
        println!("Caption: {}", post.caption);
        let config = Config {
            transparent: true,
            ..Default::default()
        };
        let response = get(&post.photo_url).expect("Failed to download image");
        let bytes = response.bytes().expect("Failed to convert response into bytes");
        let img = ImageReader::new(Cursor::new(bytes)).decode().expect("Failed to decode image");
        if let Err(e) = viuer::print(&img, &config) {
            eprintln!("Failed to display image: {}", e);
        }
        println!("\n\n");
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let users = load_usernames_from_file("usernames.txt")?;
        for user in users {
            let posts = fetch_posts_for_user(&user).await?;
            display_posts_in_terminal(posts)?;
        }
        Ok(())
    })
}
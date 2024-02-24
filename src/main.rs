use anyhow::{Ok, Result};
use askama::Template;
use estimated_read_time::Options;
use gray_matter::engine::YAML;
use gray_matter::Matter;
use minify_html::{minify, Cfg};
use serde::Deserialize;
use std::{fs, path::Path};

use walkdir::WalkDir;

const ORIGIN: &str = "https://lndev.nl";
const OUT_DIR: &str = "out";
const PUBLIC_DIR: &str = "public";
const POSTS_DIR: &str = "posts";
const DRAFTS_DIR: &str = "drafts";

#[derive(Template, Clone)]
#[template(path = "post.html")]
struct Post {
    matter: PostMatter,
    page_title: String,
    content: String,
    path: String,
    full_url: String,
    full_image_url: String,
    reading_time: String,
    twitter: Social,
    linkedin: Social,
    reddit: Social,
    facebook: Social,
    whatsapp: Social,
    telegram: Social,
}

#[derive(Clone)]
struct Social {
    url: String,
    label: String,
}
#[derive(Deserialize, Debug, Clone)]
struct PostMatter {
    title: String,
    summary: String,
    cover: CoverMatter,
    date: String,
    modified: Option<String>,
    tags: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct CoverMatter {
    image: String,
}

#[derive(Template)]
#[template(path = "blog.html")]
struct Blog {
    description: String,
    page_title: String,
    posts: Vec<Post>,
}

fn main() -> Result<()> {
    copy_dir_all(PUBLIC_DIR, OUT_DIR)?;
    let posts = collect_posts(POSTS_DIR)?;
    let drafts = collect_posts(DRAFTS_DIR)?;
    let all_posts = posts.clone().into_iter().chain(drafts.clone().into_iter());
    for post in all_posts {
        let html = post.render()?;
        let path = Path::new(OUT_DIR).join(&post.path.trim_start_matches("/"));
        write_file(&path.join("index.html"), html)?;
    }

    let blog = Blog {
        description: String::from(
            "Where insights are shared on development on the lightning network.",
        ),
        page_title: String::from("lndev - blog"),
        posts,
    };
    let blogdir = Path::new(OUT_DIR).join("blog");
    write_file(&blogdir.join("index.html"), blog.render()?)?;
    let draft_blog = Blog {
        description: String::from("Currently unfinished drafts"),
        page_title: String::from("lndev - drafts"),
        posts: drafts,
    };
    let draftdir = Path::new(OUT_DIR).join("drafts");

    write_file(&draftdir.join("index.html"), draft_blog.render()?)?;

    Ok(())
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            copy_file(&entry.path(), &dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
fn copy_file(from: &Path, to: &Path) -> Result<()> {
    let content = fs::read(from)?;
    write_file(to, &content)?;
    Ok(())
}
fn write_file<C>(path: &Path, contents: C) -> Result<()>
where
    C: AsRef<[u8]>,
{
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut modified: Option<Vec<u8>> = None;
    if let Some(extension) = path.extension() {
        modified = match extension.to_str().unwrap() {
            "html" => Some(minify_html(contents.as_ref())),
            // "css" => minify_css(contents)?,
            _ => None,
        };
    }
    if let Some(modified) = modified {
        fs::write(path, modified)?;
    } else {
        fs::write(path, contents)?;
    }

    Ok(())
}

fn minify_html(contents: &[u8]) -> Vec<u8> {
    let cfg = Cfg {
        do_not_minify_doctype: true,
        ensure_spec_compliant_unquoted_attribute_values: true,
        keep_spaces_between_attributes: true,
        keep_closing_tags: true,
        ..Cfg::default()
    };
    minify(contents, &cfg)
}

fn collect_posts(dir: impl AsRef<Path>) -> Result<Vec<Post>> {
    let mut posts = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.path().to_str().map(|e| String::from(e)))
        .filter(|e| e.ends_with(".md"))
        .map(|p| get_post(Path::new(&p)))
        .collect::<Result<Vec<Post>>>()?;
    posts.sort_by(|a, b| b.matter.date.cmp(&a.matter.date));
    Ok(posts)
}

fn get_post(relative_path: &Path) -> Result<Post> {
    let slug = relative_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let mut path = relative_path
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
        + "/"
        + &slug;
    if !path.starts_with("/") {
        path.insert_str(0, "/");
    }
    let origin = String::from(ORIGIN);
    let full_url = origin.clone() + &path;
    let file_content = fs::read_to_string(relative_path)?;
    let parsed = Matter::<YAML>::new()
        .parse_with_struct::<PostMatter>(&file_content)
        .unwrap();

    let content = markdown::to_html(&parsed.content);
    let read_time_seconds =
        estimated_read_time::text(&parsed.content, &Options::new().build().unwrap()).seconds();
    let read_time = if read_time_seconds < 60 {
        format!("{} sec read", read_time_seconds)
    } else {
        format!("{} min read", read_time_seconds / 60)
    };
    let encoded_title: String =
        url::form_urlencoded::byte_serialize(parsed.data.title.as_bytes()).collect();
    let encoded_url: String = url::form_urlencoded::byte_serialize(full_url.as_bytes()).collect();
    let encoded_tags: String =
        url::form_urlencoded::byte_serialize(parsed.data.tags.join(",").as_bytes()).collect();
    let lbl = |name| format!("Share {} on {}", parsed.data.title, name);
    Ok(Post {
        matter: parsed.data.clone(),
        page_title: String::from("lndev - ") + &parsed.data.title,
        content,
        path,
        full_url,
        full_image_url: origin.clone() + &parsed.data.cover.image,
        reading_time: read_time,
        twitter: Social {
            url: format!(
                "https://x.com/intent/tweet/?text={}&url={}&hashtags={}",
                encoded_title, encoded_url, encoded_tags
            ),
            label: lbl("X"),
        },
        facebook: Social {
            url: format!("https://facebook.com/sharer/sharer.php?u={}", encoded_url),
            label: lbl("Facebook"),
        },
        linkedin: Social {
            url: format!("https://www.linkedin.com/shareArticle?mini=true&url={}&title={}&summary={}&source={}", encoded_url, encoded_title, encoded_title, encoded_url),
            label: lbl("LinkedIn"),
        },
        reddit: Social {
            url: format!("https://reddit.com/submit?url={}&title={}", encoded_url, encoded_title),
            label: lbl("Reddit"),
        },
        whatsapp: Social {
            url: format!("https://api.whatsapp.com/send?text={}%20-%20{}", encoded_title, encoded_url),
            label: lbl("WhatsApp"),
        },
        telegram: Social {
            url: format!("https://telegram.me/share/url?text={}&url={}", encoded_title, encoded_url),
            label: lbl("Telegram"),
        },
    })
}

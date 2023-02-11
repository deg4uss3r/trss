use atom_syndication::Feed;
use rss::Channel;
use std::{cmp::Ordering, error::Error};

#[derive(Debug)]
enum FeedType {
    Rss(Channel),
    Atom(Feed),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Website {
    pub name: String,
    pub uri: String,
    pub author: String,
    pub updated_at: String,
    pub articles: Vec<Article>,
}

impl Ord for Website {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Website {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Article {
    pub title: String,
    pub subtitle: Option<String>,
    pub updated_at: String,
    pub content: String,
}

impl Ord for Article {
    fn cmp(&self, other: &Self) -> Ordering {
        self.title.cmp(&other.title)
    }
}

impl PartialOrd for Article {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub(crate) fn example_feed<'a>(url: &str) -> Result<Website, Box<dyn Error>> {
    let content = reqwest::blocking::get(url)?.text()?;

    let feed: FeedType = match Channel::read_from(content.as_bytes()) {
        Ok(c) => FeedType::Rss(c),
        Err(_) => FeedType::Atom(Feed::read_from(content.as_bytes()).unwrap()),
    };

    Ok(match feed {
        FeedType::Rss(content) => Website {
            name: content.title().to_string(),
            uri: content.link().to_string(),
            author: content.managing_editor().unwrap_or("N/A").to_string(),
            updated_at: content.last_build_date().unwrap_or("N/A").to_string(),
            articles: content
                .items()
                .iter()
                .map(|item| Article {
                    title: item.title().unwrap_or("Untitled").to_string(),
                    subtitle: Some(item.description().unwrap_or("").to_string()),
                    updated_at: item.pub_date().unwrap_or("N/A").to_string(),
                    content: item.content().unwrap_or("N/A").to_string(),
                })
                .collect(),
        },
        FeedType::Atom(content) => Website {
            name: content.title().value.clone(),
            uri: content
                .links()
                .iter()
                .map(|link| link.href.clone())
                .collect::<Vec<String>>()
                .join(" "),
            author: content
                .authors()
                .iter()
                .map(|author| author.name.clone())
                .collect::<Vec<String>>()
                .join(" "),
            updated_at: content.updated().to_string(),
            articles: content
                .entries()
                .iter()
                .map(|item| Article {
                    title: item.title().value.clone(),
                    subtitle: Some(item.summary().cloned().unwrap_or_default().value),
                    updated_at: item.published().unwrap().to_string(),
                    content: item
                        .content()
                        .cloned()
                        .unwrap_or_default()
                        .value
                        .unwrap_or("N/A".to_string())
                        .to_string(),
                })
                .collect(),
        },
    })
}

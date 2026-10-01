//! The site's pages, rendered from the templates in `templates/` and the data in `content/`.
//!
//! The data files are compiled into the binary and parsed when the server starts, so a typo in
//! them stops the server (and `cargo test`) instead of showing a broken page.

use askama::Template;
use serde::Deserialize;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Home {
    pub page: PageMeta,
    pub about: About,
    pub research: Research,
    pub medicine: Medicine,
    pub community: Community,
    pub teaching: Teaching,
    pub contact: Contact,
    pub footer: Footer,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageMeta {
    pub title: String,
    pub description: String,
    pub keywords: String,
    pub og_description: String,
    pub tagline: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct About {
    pub intro: Vec<String>,
    pub keyfacts: Vec<String>,
    pub background: Vec<String>,
    pub exchange: Vec<String>,
    pub education: Vec<String>,
    pub honors: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Research {
    pub publications_more: String,
    pub publications: Vec<Publication>,
    pub projects: Vec<Project>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Publication {
    pub year: String,
    pub title: String,
    /// Empty when the paper has no public link yet.
    #[serde(default)]
    pub url: String,
    pub venue: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub title: String,
    pub description: String,
    pub links: Vec<Link>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub label: String,
    pub url: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Medicine {
    pub intro: String,
    pub notes: Vec<Note>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Note {
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub note: String,
}

impl Note {
    /// Links to other sites open in a new tab; pages on this site do not.
    pub fn is_external(&self) -> bool {
        self.url.starts_with("http")
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Community {
    pub student: Vec<String>,
    pub student_link: Link,
    pub tech: Vec<String>,
    pub tech_link: Link,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Teaching {
    pub items: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contact {
    pub lines: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Footer {
    pub lines: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NotFoundData {
    facts: Vec<String>,
    gifs: Vec<String>,
}

static HOME: LazyLock<Home> = LazyLock::new(|| {
    toml::from_str(include_str!("../content/home.toml"))
        .unwrap_or_else(|err| panic!("content/home.toml: {err}"))
});

static NOT_FOUND: LazyLock<NotFoundData> = LazyLock::new(|| {
    let data: NotFoundData = toml::from_str(include_str!("../content/not_found.toml"))
        .unwrap_or_else(|err| panic!("content/not_found.toml: {err}"));
    assert!(
        !data.facts.is_empty(),
        "content/not_found.toml: facts is empty"
    );
    assert!(
        data.gifs.len() == 3,
        "content/not_found.toml: the page has three GIF slots"
    );
    data
});

/// When the deployed commit was made, shown in the homepage footer. scripts/deploy.sh passes it
/// to the image as SITE_UPDATED; it is empty in local builds.
static UPDATED: LazyLock<String> =
    LazyLock::new(|| std::env::var("SITE_UPDATED").unwrap_or_default());

#[derive(Template)]
#[template(path = "index.html")]
struct IndexPage<'a> {
    home: &'a Home,
    year: i64,
    updated: &'a str,
}

#[derive(Template)]
#[template(path = "pe.html")]
struct PePage {
    year: i64,
}

#[derive(Template)]
#[template(path = "404.html")]
struct NotFoundPage<'a> {
    path: &'a str,
    fact: &'a str,
    gifs: Vec<&'a str>,
}

/// Parse every data file now, so that bad data fails at startup.
pub fn load() {
    LazyLock::force(&HOME);
    LazyLock::force(&NOT_FOUND);
    LazyLock::force(&UPDATED);
}

pub fn home() -> askama::Result<String> {
    IndexPage {
        home: &HOME,
        year: current_year(),
        updated: &UPDATED,
    }
    .render()
}

pub fn pe() -> askama::Result<String> {
    PePage {
        year: current_year(),
    }
    .render()
}

/// The 404 page shows the missing path, a random fact, and the three GIFs in a random order.
pub fn not_found(path: &str) -> askama::Result<String> {
    let facts = &NOT_FOUND.facts;
    let mut gifs: Vec<&str> = NOT_FOUND.gifs.iter().map(String::as_str).collect();
    for i in (1..gifs.len()).rev() {
        gifs.swap(i, random_below(i + 1));
    }
    NotFoundPage {
        path,
        fact: &facts[random_below(facts.len())],
        gifs,
    }
    .render()
}

/// A random number below `n`. The std hasher's keys are random per instance, which is plenty
/// for picking a fact; nothing here needs to be unpredictable.
fn random_below(n: usize) -> usize {
    use std::hash::{BuildHasher, Hasher};
    let value = std::collections::hash_map::RandomState::new()
        .build_hasher()
        .finish();
    (value % n as u64) as usize
}

/// The current year in Taiwan (UTC+8), for the copyright line.
fn current_year() -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    year_of(now + 8 * 3600)
}

/// The calendar year of a Unix timestamp, using Howard Hinnant's civil-from-days algorithm.
fn year_of(unix_seconds: i64) -> i64 {
    let days = unix_seconds.div_euclid(86_400) + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let year = year_of_era + era * 400;
    if month_index >= 10 { year + 1 } else { year }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_page_renders() {
        load();
        assert!(
            home()
                .unwrap()
                .contains("<title>蔡秀吉 Tsai Hsiu-Chi — 學術與專案首頁</title>")
        );
        assert!(pe().unwrap().contains("肺栓塞 Pulmonary Embolism (PE)"));
        assert!(
            not_found("/missing")
                .unwrap()
                .contains("<code id=\"url\">/missing</code>")
        );
    }

    #[test]
    fn not_found_escapes_the_path() {
        let page = not_found("/<script>alert(1)</script>").unwrap();
        assert!(!page.contains("<script>alert(1)</script>"));
        assert!(page.contains("&#60;script&#62;alert(1)&#60;/script&#62;"));
    }

    #[test]
    fn year_changes_at_midnight_taiwan_time() {
        assert_eq!(year_of(0), 1970);
        // 2026-12-31T15:59:59Z is still 2026 in Taiwan; one second later it is 2027.
        assert_eq!(year_of(1_798_732_799 + 8 * 3600), 2026);
        assert_eq!(year_of(1_798_732_800 + 8 * 3600), 2027);
        // Leap day.
        assert_eq!(year_of(1_709_164_800), 2024);
    }
}

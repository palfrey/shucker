#![allow(unused_doc_comments)]

mod rules;
use anyhow::Result;

pub fn shuck(url: &str) -> Result<String> {
    rules::stripper(url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn no_change() {
        let original = "https://arstechnica.com/?p=2053037";
        let result = shuck(original).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn strip_google_ads() {
        let original = "https://www.businessinsider.com/best-modern-christmas-songs-2024?utm_source=pocket_discover";
        let result = shuck(original).unwrap();
        assert_eq!(
            result,
            "https://www.businessinsider.com/best-modern-christmas-songs-2024"
        );
    }

    #[test]
    fn strip_simple_url_filter() {
        // Ones like '||slack.com/downloads/'
        let original = "https://slack.com/downloads/linux?t=[Slack channel ID]";
        let result = shuck(original).unwrap();
        assert_eq!(result, "https://slack.com/downloads/linux");
    }

    #[test]
    fn domain_filters() {
        let original =
            "https://www6.nhk.or.jp/nhkpr/?cid=prhk-carousel-berabou&cid=jp-g-pr-carousel3";
        let result = shuck(original).unwrap();
        assert_eq!(result, "https://www6.nhk.or.jp/nhkpr/");
    }
}

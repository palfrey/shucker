#![allow(unused_doc_comments)]

mod rules;
use anyhow::Result;

pub fn shuck(url: &str) -> Result<String> {
    rules::stripper(url)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // Ones like '||bing.'
        let original = "https://www.bing.com/search?q=hello&form=QBLH&sp=-1&pq=hello&sc=8-5&qs=n&sk=&cvid=49B83A335B1C4884B71B2FAD4A8027A9";
        let result = shuck(original).unwrap();
        assert_eq!(
            result,
            "https://www.bing.com/search?q=hello&form=QBLH&sc=8-5&sk="
        );
    }

    #[test]
    fn domain_filters() {
        let original = "https://fortune.com/2021/03/11/amazon-fresh-retail-stores-vacant-plan/?queryly=related_article";
        let result = shuck(original).unwrap();
        assert_eq!(
            result,
            "https://fortune.com/2021/03/11/amazon-fresh-retail-stores-vacant-plan/"
        );
    }
}

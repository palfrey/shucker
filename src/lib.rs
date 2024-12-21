pub fn shuck(url: &str) -> String {
    String::from(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_change() {
        let original = "https://arstechnica.com/?p=2053037";
        let result = shuck(original);
        assert_eq!(result, original);
    }

    #[test]
    fn strip_google_ads() {
        let original = "https://www.businessinsider.com/best-modern-christmas-songs-2024?utm_source=pocket_discover";
        let result = shuck(original);
        assert_eq!(
            result,
            "https://www.businessinsider.com/best-modern-christmas-songs-2024"
        );
    }
}

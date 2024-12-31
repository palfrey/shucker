#![allow(unused_doc_comments)]

mod rules;
use anyhow::Result;

#[cfg(feature = "python")]
use pyo3::{
    exceptions::PyRuntimeError,
    pyfunction, pymodule,
    types::{PyModule, PyModuleMethods},
    wrap_pyfunction, Bound, PyResult,
};

pub fn shuck(url: &str) -> Result<String> {
    rules::stripper(url)
}

#[cfg(feature = "python")]
#[pyfunction(name = "shuck")]
fn py_shuck(url: &str) -> PyResult<String> {
    rules::stripper(url).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[cfg(feature = "python")]
#[pymodule]
fn shucker(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_shuck, m)?)?;
    Ok(())
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

    #[test]
    fn regex_remove_param() {
        let original ="https://www.reddit.com/r/apple/comments/1g478w1/apple_announces_new_ipad_mini_with_a17_pro_chip/?%24deep_link=true&post_index=1&%243p=e_as";
        let result = shuck(original).unwrap();
        assert_eq!(result, "https://www.reddit.com/r/apple/comments/1g478w1/apple_announces_new_ipad_mini_with_a17_pro_chip/");
    }

    #[test]
    fn regex_remove_param_on_host() {
        // Like regex_remove_param but specifically with a host that doesn't end "reddit.com"
        let original ="https://www.reddit-not.com/r/apple/comments/1g478w1/apple_announces_new_ipad_mini_with_a17_pro_chip/?%24deep_link=true&post_index=1&%243p=e_as";
        let result = shuck(original).unwrap();
        assert_eq!(result, "https://www.reddit-not.com/r/apple/comments/1g478w1/apple_announces_new_ipad_mini_with_a17_pro_chip/?%24deep_link=true&post_index=1&%243p=e_as");
    }
}

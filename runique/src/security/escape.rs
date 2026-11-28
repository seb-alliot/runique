//! Text made safe to write into HTML by hand, where no template engine
//! escapes it: the fallback error pages, a render error shown in a form.

pub(crate) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::html_escape;

    #[test]
    fn test_html_escape_ampersand() {
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }

    #[test]
    fn test_html_escape_lt_gt() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
    }

    #[test]
    fn test_html_escape_double_quote() {
        assert_eq!(html_escape(r#"say "hi""#), "say &quot;hi&quot;");
    }

    #[test]
    fn test_html_escape_single_quote() {
        assert_eq!(html_escape("it's"), "it&#x27;s");
    }

    #[test]
    fn test_html_escape_empty() {
        assert_eq!(html_escape(""), "");
    }

    #[test]
    fn test_html_escape_mixed() {
        assert_eq!(
            html_escape("<b>\"Hello\" & 'World'</b>"),
            "&lt;b&gt;&quot;Hello&quot; &amp; &#x27;World&#x27;&lt;/b&gt;"
        );
    }
}

use url::Url;

pub fn build_target_url(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return "about:blank".to_string();
    }

    if trimmed == "about:blank" || trimmed == "about:midlevel" {
        return trimmed.to_string();
    }

    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("file://")
    {
        return trimmed.to_string();
    }

    if is_likely_url(trimmed) {
        return format!("https://{}", trimmed);
    }

    let encoded = url::form_urlencoded::byte_serialize(trimmed.as_bytes()).collect::<String>();
    format!("https://www.google.com/search?q={}", encoded)
}

fn is_likely_url(input: &str) -> bool {
    if input.contains(' ') {
        return false;
    }

    let test_url = format!("https://{}", input);
    if let Ok(parsed) = Url::parse(&test_url) {
        if let Some(host) = parsed.host_str() {
            if host.contains('.') || host == "localhost" || host.parse::<std::net::IpAddr>().is_ok() {
                return true;
            }
        }
    }
    false
}
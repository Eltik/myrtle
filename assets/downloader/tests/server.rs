use downloader::error::DownloaderError;
use downloader::server::Server;

#[test]
fn parse_all_aliases() {
    let cases: &[(&str, Server)] = &[
        ("official", Server::Official),
        ("cn", Server::Official),
        ("bilibili", Server::Bilibili),
        ("bili", Server::Bilibili),
        ("b", Server::Bilibili),
        ("en", Server::En),
        ("global", Server::En),
        ("jp", Server::Jp),
        ("japan", Server::Jp),
        ("kr", Server::Kr),
        ("korea", Server::Kr),
        ("tw", Server::Tw),
        ("taiwan", Server::Tw),
    ];

    for (input, expected) in cases {
        let parsed: Server = input
            .parse()
            .unwrap_or_else(|_| panic!("failed to parse {input:?}"));
        assert_eq!(parsed, *expected, "input: {input:?}");
    }
}

#[test]
fn parse_case_insensitive() {
    let cases = [
        "OFFICIAL", "Official", "CN", "En", "JP", "BILIBILI", "Taiwan",
    ];
    for input in cases {
        let result: Result<Server, _> = input.parse();
        assert!(result.is_ok(), "failed to parse {input:?}");
    }
}

#[test]
fn parse_invalid() {
    let cases = ["invalid", "", "xyz", "us", "china"];
    for input in cases {
        let result: Result<Server, DownloaderError> = input.parse();
        assert!(result.is_err(), "expected error for {input:?}");
        assert!(
            matches!(result.unwrap_err(), DownloaderError::InvalidServer(_)),
            "expected InvalidServer for {input:?}"
        );
    }
}

#[test]
fn display_roundtrip() {
    let variants = [
        Server::Official,
        Server::Bilibili,
        Server::En,
        Server::Jp,
        Server::Kr,
        Server::Tw,
    ];

    for server in variants {
        let display = server.to_string();
        let parsed: Server = display.parse().unwrap();
        assert_eq!(parsed, server, "roundtrip failed for {server:?}");
    }
}

#[test]
fn version_urls_contain_expected_domains() {
    assert!(Server::Official.version_url().contains("hypergryph"));
    assert!(Server::Bilibili.version_url().contains("hypergryph"));
    assert!(Server::En.version_url().contains("ark-us"));
    assert!(Server::Jp.version_url().contains("ark-jp"));
    assert!(Server::Kr.version_url().contains("ark-kr"));
    assert!(Server::Tw.version_url().contains("ark-tw"));
}

#[test]
fn cdn_urls_contain_expected_domains() {
    assert!(Server::Official.cdn_base_url().contains("hycdn.cn"));
    assert!(Server::Bilibili.cdn_base_url().contains("hycdn.cn"));
    assert!(Server::En.cdn_base_url().contains("ark-us"));
    assert!(Server::Jp.cdn_base_url().contains("ark-jp"));
    assert!(Server::Kr.cdn_base_url().contains("ark-kr"));
    assert!(Server::Tw.cdn_base_url().contains("ark-tw"));
}

#[test]
fn all_urls_are_https() {
    let variants = [
        Server::Official,
        Server::Bilibili,
        Server::En,
        Server::Jp,
        Server::Kr,
        Server::Tw,
    ];

    for server in variants {
        assert!(
            server.version_url().starts_with("https://"),
            "{server:?} version_url not HTTPS"
        );
        assert!(
            server.cdn_base_url().starts_with("https://"),
            "{server:?} cdn_base_url not HTTPS"
        );
    }
}

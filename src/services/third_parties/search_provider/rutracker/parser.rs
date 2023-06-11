use crate::types::TopicId;
use scraper::error::SelectorErrorKind;
use scraper::{Html, Selector};

#[derive(Debug, thiserror::Error)]
pub(crate) enum ParseError {
    #[error(transparent)]
    SelectorError(#[from] SelectorErrorKind<'static>),
}

#[derive(Debug, PartialEq)]
pub(crate) struct SearchResult {
    pub(crate) title: String,
    pub(crate) topic_id: TopicId,
    pub(crate) seeds_number: u64,
}

const AUDIO_FORMAT_PRIORITY: [&str; 4] = ["FLAC", "MP3", "ALAC", "AAC"];
const AUDIO_BITRATE_PRIORITY: [&str; 3] = ["lossless", "320 kbps", "256 kbps"];

fn get_search_result_priority(result: &SearchResult) -> usize {
    let format_priority = AUDIO_FORMAT_PRIORITY
        .iter()
        .enumerate()
        .find_map(|(i, format)| {
            if result.title.contains(format) {
                Some(i)
            } else {
                None
            }
        })
        .unwrap_or(10);
    let bitrate_priority = AUDIO_BITRATE_PRIORITY
        .iter()
        .enumerate()
        .find_map(|(i, bitrate)| {
            if result.title.contains(bitrate) {
                Some(i)
            } else {
                None
            }
        })
        .unwrap_or(10);
    let seeds_priority = match result.seeds_number {
        x if x == 0 => 10,
        x if x < 10 => 3,
        x if x < 20 => 2,
        x if x < 30 => 1,
        _ => 0,
    };

    format_priority * 5 + bitrate_priority * 10 + seeds_priority
}

pub(crate) fn parse_search_results(raw_html: &str) -> Result<Vec<SearchResult>, ParseError> {
    let html = Html::parse_document(raw_html);

    let table_row_selector = Selector::parse(r#"table.forumline tr"#)?;
    let table_entries = html.select(&table_row_selector);

    let href_selector = Selector::parse(r#"a[href]"#)?;
    let td_selector = Selector::parse(r#"td"#)?;
    let seeds_selector = Selector::parse(r#"b.seedmed"#)?;

    let mut results: Vec<_> = table_entries
        .skip(1)
        .filter(|el| el.children().filter(|el| el.value().is_element()).count() == 10)
        .filter_map(|el| {
            let columns = el.select(&td_selector).collect::<Vec<_>>();
            let link = columns[3].select(&href_selector).next()?;
            let category_str = columns[2]
                .select(&href_selector)
                .next()?
                .inner_html()
                .to_lowercase();

            if !category_str.contains("loss") {
                return None;
            }

            let title = link.inner_html().to_string();
            let topic_id = link
                .value()
                .attr("data-topic_id")?
                .to_string()
                .parse::<u64>()
                .ok()?
                .into();
            let seeds_number = columns[6]
                .select(&seeds_selector)
                .next()?
                .inner_html()
                .to_string()
                .parse()
                .ok()?;

            Some(SearchResult {
                title,
                topic_id,
                seeds_number,
            })
        })
        .filter(|r| !r.title.contains("image+.cue"))
        .collect();

    results.sort_by(|a, b| get_search_result_priority(a).cmp(&get_search_result_priority(b)));

    Ok(results)
}

#[derive(Debug, PartialEq)]
pub(crate) struct Topic {
    pub(crate) torrent_id: i64,
}

pub(crate) fn parse_topic(raw_html: &str) -> Result<Option<Topic>, ParseError> {
    let html = Html::parse_document(raw_html);

    let download_link_selector = Selector::parse(r#"table.attach tr td a.dl-link"#)?;
    let mut download_link = html.select(&download_link_selector);

    let topic = download_link.next().and_then(|el| {
        let download_id = el
            .value()
            .attr("href")?
            .to_string()
            .replace("dl.php?t=", "")
            .parse()
            .ok()?;

        Some(Topic {
            torrent_id: download_id,
        })
    });

    Ok(topic)
}

const CAPTCHA_IS_REQUIRED_TEXT: &str = "введите код подтверждения";
const INCORRECT_PASSWORD_TEXT: &str = "неверный пароль";
const SUCCESSFUL_LOGIN_TEXT: &str = "log-out-icon";

#[derive(Debug, thiserror::Error)]
pub(crate) enum AuthError {
    #[error("Captcha verification is required.")]
    CaptchaVerificationIsRequired,
    #[error("Incorrect login or password.")]
    IncorrectPasswordText,
    #[error("Unknown authentication error")]
    UnknownAuthError,
}

pub(crate) fn parse_and_validate_auth_state(raw_html: &str) -> Result<(), AuthError> {
    if raw_html.contains(CAPTCHA_IS_REQUIRED_TEXT) {
        return Err(AuthError::CaptchaVerificationIsRequired);
    }

    if raw_html.contains(INCORRECT_PASSWORD_TEXT) {
        return Err(AuthError::IncorrectPasswordText);
    }

    if !raw_html.contains(SUCCESSFUL_LOGIN_TEXT) {
        return Err(AuthError::UnknownAuthError);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_search_results() {
        let results = parse_search_results(include_str!("fixtures/search_results.html"))
            .expect("Expected successful parse results");

        let expected_results = vec![
            SearchResult { title: "(Trance, Dream House, Downtempo) Robert Miles - Dreamland - 1996 (Deconstruction [74321 42974 2]), FLAC (tracks+.cue), lossless".into(), topic_id: 1183770.into(), seeds_number: 18 },
            SearchResult { title: "(Trance, Dream House, Downtempo) Robert Miles - Dreamland (The Winter Edition) - 1996 (Urban #533 791-2), FLAC (tracks+.cue), lossless".into(), topic_id: 1184081.into(), seeds_number: 11 },
            SearchResult { title: "(Trance) [WEB] Robert Miles - Dreamland (Remastered) - 2016, FLAC (tracks), lossless".into(), topic_id: 5318721.into(), seeds_number: 8 },
            SearchResult { title: "(Trance, Dream House, Downtempo) Robert Miles - Dreamland - 1996, FLAC (tracks+.cue) lossless".into(), topic_id: 3418878.into(), seeds_number: 4 },
            SearchResult { title: "Robert Miles - Dreamland - 1996, ALAC, lossless".into(), topic_id: 1201152.into(), seeds_number: 3 },
            SearchResult { title: "(Trance) Robert Miles - Dreamland (Remastered) - 2016, MP3, 320 kbps".into(), topic_id: 5309922.into(), seeds_number: 9 },
            SearchResult { title: "(Dream House) Robert Miles - Dreamland (Including One and One) - 1996 [WEB], AAC (tracks) 256 kbps".into(), topic_id: 4737164.into(), seeds_number: 2 },
        ];

        assert_eq!(7, results.len());
        assert_eq!(expected_results, results);
    }

    #[test]
    fn test_parse_topic() {
        let parsed_topic = parse_topic(include_str!("fixtures/topic.html"))
            .expect("Expected successful parse results");

        assert_eq!(
            Some(Topic {
                torrent_id: 5309922
            }),
            parsed_topic
        );
    }
}

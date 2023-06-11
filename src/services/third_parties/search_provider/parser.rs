use crate::types::TopicId;
use scraper::{Html, Selector};

#[derive(Debug, thiserror::Error)]
pub(crate) enum ParseError {
    #[error("Unable to select table with search results")]
    TableSelectorError,
    #[error("Unexpected")]
    Unexpected,
}

#[derive(Debug, PartialEq)]
pub(crate) struct SearchResult {
    pub(crate) title: String,
    pub(crate) topic_id: TopicId,
    pub(crate) seeds_number: u64,
}

fn get_search_result_priority(result: &SearchResult) -> usize {
    match result {
        res if res.title.contains("tracks")
            && res.title.contains("FLAC (tracks)")
            && res.seeds_number > 1 =>
        {
            1
        }
        res if res.title.contains("tracks")
            && res.title.contains("MP3, 320 kbps")
            && res.seeds_number > 1 =>
        {
            2
        }
        res if res.title.contains("tracks")
            && res.title.contains("AAC")
            && res.seeds_number > 1 =>
        {
            3
        }
        res if res.seeds_number > 1 => 4,
        _ => 10,
    }
}

pub(crate) fn parse_search_results(raw: &str) -> Result<Vec<SearchResult>, ParseError> {
    let html = Html::parse_document(raw);

    let table_row_selector =
        Selector::parse(r#"table.forumline tr"#).map_err(|_| ParseError::TableSelectorError)?;
    let table_entries = html.select(&table_row_selector);

    let href_selector = &Selector::parse(r#"a[href]"#).unwrap();
    let td_selector = &Selector::parse(r#"td"#).unwrap();
    let seeds_selector = &Selector::parse(r#"b.seedmed"#).unwrap();

    let mut results: Vec<_> = table_entries
        .skip(1)
        .filter(|el| el.children().filter(|el| el.value().is_element()).count() == 10)
        .filter_map(|el| {
            let columns = el.select(&td_selector).collect::<Vec<_>>();
            let link = columns[3].select(&href_selector).next()?;

            let title = link.inner_html().to_string();

            let topic_id_raw = link.value().attr("href")?.to_string();
            let topic_id = topic_id_raw
                .replace("viewtopic.php?t=", "")
                .parse::<u64>()
                .ok()?
                .into();

            let seeds = columns[6].select(&seeds_selector).next()?;
            let seeds_number = seeds.inner_html().to_string().parse().ok()?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_search_results() {
        let results = parse_search_results(include_str!("fixtures/search_results.html"))
            .expect("Expected successful parse results");

        let expected_results = vec![
            SearchResult { title: "(Trance) [WEB] Robert Miles - Dreamland (Remastered) - 2016, FLAC (tracks), lossless".into(), topic_id: 5318721.into(), seeds_number: 8 },
            SearchResult { title: "(Trance) Robert Miles - Dreamland (Remastered) - 2016, MP3, 320 kbps".into(), topic_id: 5309922.into(), seeds_number: 9 },
            SearchResult { title: "(Trance, Dream House, Downtempo) Robert Miles - Dreamland - 1996 (Deconstruction [74321 42974 2]), FLAC (tracks+.cue), lossless".into(), topic_id: 1183770.into(), seeds_number: 18 }, 
            SearchResult { title: "(Trance, Dream House, Downtempo) Robert Miles - Dreamland (The Winter Edition) - 1996 (Urban #533 791-2), FLAC (tracks+.cue), lossless".into(), topic_id: 1184081.into(), seeds_number: 11 }, 
            SearchResult { title: "(Dream House) Robert Miles - Dreamland (Including One and One) - 1996 [WEB], AAC (tracks) 256 kbps".into(), topic_id: 4737164.into(), seeds_number: 2 }, 
            SearchResult { title: "(Trance, Dream House, Downtempo) Robert Miles - Dreamland - 1996, FLAC (tracks+.cue) lossless".into(), topic_id: 3418878.into(), seeds_number: 4 }, 
            SearchResult { title: "Robert Miles - Dreamland - 1996, ALAC, lossless".into(), topic_id: 1201152.into(), seeds_number: 3 }, 
        ];

        assert_eq!(10, results.len());
        assert_eq!(expected_results, results);
    }
}

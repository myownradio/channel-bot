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

pub(crate) fn parse_search_results(raw: &str) -> Result<Vec<SearchResult>, ParseError> {
    let html = Html::parse_document(raw);

    let table_row_selector =
        Selector::parse(r#"table.forumline tr"#).map_err(|_| ParseError::TableSelectorError)?;
    let table_entries = html.select(&table_row_selector);

    let href_selector = &Selector::parse(r#"a[href]"#).unwrap();
    let td_selector = &Selector::parse(r#"td"#).unwrap();
    let seeds_selector = &Selector::parse(r#"b.seedmed"#).unwrap();

    Ok(table_entries
        .skip(1)
        .filter(|el| el.children().filter(|el| el.value().is_element()).count() == 10)
        .map(|el| {
            let columns = el.select(&td_selector).collect::<Vec<_>>();
            let link = columns[3].select(&href_selector).next().unwrap();

            let title = link.inner_html().to_string();
            let topic_id_raw = link.value().attr("href").unwrap_or_default().to_string();

            let seeds = columns[6].select(&seeds_selector).next().unwrap();

            let topic_id = topic_id_raw
                .replace("viewtopic.php?t=", "")
                .parse::<u64>()
                .unwrap()
                .into();

            let seeds_number = seeds.inner_html().to_string().parse().unwrap();

            SearchResult {
                title,
                topic_id,
                seeds_number,
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_search_results() {
        let results = parse_search_results(include_str!("fixtures/search_results.html"))
            .expect("Expected successful parse results");

        let expected_results = vec![
            SearchResult { title: "(Dream-House/Dance/Trance) [LP] [24/96] Robert Miles - Dreamland - 2016 (1996), FLAC (image+.cue)".into(), topic_id: 6213241.into(), seeds_number: 6 },
            SearchResult { title: "(Dream-House/Dance/Trance) [2xLP's] [24/192] Robert Miles - Dreamland (Exclusive in Russia) © 2019 (1996), WavPack (image+.cue)".into(), topic_id: 5744476.into(), seeds_number: 6 }, 
            SearchResult { title: "(Trance, Dream House, Downtempo) Robert Miles - Dreamland - 1996 (Deconstruction [74321 42974 2]), FLAC (tracks+.cue), lossless".into(), topic_id: 1183770.into(), seeds_number: 18 }, 
            SearchResult { title: "(Trance, Dream House, Downtempo) Robert Miles - Dreamland (The Winter Edition) - 1996 (Urban #533 791-2), FLAC (tracks+.cue), lossless".into(), topic_id: 1184081.into(), seeds_number: 11 }, 
            SearchResult { title: "(Trance) [WEB] Robert Miles - Dreamland (Remastered) - 2016, FLAC (tracks), lossless".into(), topic_id: 5318721.into(), seeds_number: 8 }, 
            SearchResult { title: "(Trance) Robert Miles - Dreamland (Remastered) - 2016, MP3, 320 kbps".into(), topic_id: 5309922.into(), seeds_number: 9 }, 
            SearchResult { title: "[DTSCD][UP] Robert Miles - Dreamland - 1996 (Trance, Electronic, Ambient, House)".into(), topic_id: 5217604.into(), seeds_number: 5 }, 
            SearchResult { title: "(Dream House) Robert Miles - Dreamland (Including One and One) - 1996 [WEB], AAC (tracks) 256 kbps".into(), topic_id: 4737164.into(), seeds_number: 2 }, 
            SearchResult { title: "[DTSCD][UP] Robert Miles - Dreamland - 1996, (Electronica, alternative, dance, ambient)".into(), topic_id: 3901716.into(), seeds_number: 4 }, 
            SearchResult { title: "(Trance, Dream House, Downtempo) Robert Miles - Dreamland - 1996, FLAC (tracks+.cue) lossless".into(), topic_id: 3418878.into(), seeds_number: 4 }, 
            SearchResult { title: "(Trance) Robert Miles - Dreamland - 1996 (Urban, 533 002-2), FLAC (image+.cue) lossless".into(), topic_id: 3199643.into(), seeds_number: 10 }, 
            SearchResult { title: "(Trance, Dream House, Downtempo) Robert Miles - Dreamland~New Edition (Japan) - 1996, FLAC (image+.cue), lossless".into(), topic_id: 2495343.into(), seeds_number: 7 }, 
            SearchResult { title: "(Фортепиано, Голос, Аккорды) Robert Miles - Dreamland (Children, Fable и др.) [1997, PDF, ENG]".into(), topic_id: 1295160.into(), seeds_number: 2 }, 
            SearchResult { title: "Robert Miles - Dreamland - 1996, ALAC, lossless".into(), topic_id: 1201152.into(), seeds_number:3 }, 
            SearchResult { title: "(Trance, Dreamhouse) Robert Miles - Dreamland - 1996, APE (image+.cue), lossless".into(), topic_id: 1182981.into(), seeds_number: 2 }
        ];

        assert_eq!(15, results.len());
        assert_eq!(expected_results, results);
    }
}

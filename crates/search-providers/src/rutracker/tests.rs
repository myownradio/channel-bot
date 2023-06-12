use crate::rutracker::parser::{parse_search_results, parse_topic};
use crate::{DownloadId, SearchResult, Topic, TopicId};

#[test]
fn test_parsing_of_search_results() {
    let results = parse_search_results(include_str!("fixtures/search_results.html"))
        .expect("Expected successful parse results");

    let expected_results = vec![
        SearchResult { title: "(Trance, Dream House, Downtempo) Robert Miles - Dreamland - 1996 (Deconstruction [74321 42974 2]), FLAC (tracks+.cue), lossless".into(), topic_id: TopicId(1183770), seeds_number: 18 },
        SearchResult { title: "(Trance, Dream House, Downtempo) Robert Miles - Dreamland (The Winter Edition) - 1996 (Urban #533 791-2), FLAC (tracks+.cue), lossless".into(), topic_id: TopicId(1184081), seeds_number: 11 },
        SearchResult { title: "(Trance) [WEB] Robert Miles - Dreamland (Remastered) - 2016, FLAC (tracks), lossless".into(), topic_id: TopicId(5318721), seeds_number: 8 },
        SearchResult { title: "(Trance, Dream House, Downtempo) Robert Miles - Dreamland - 1996, FLAC (tracks+.cue) lossless".into(), topic_id: TopicId(3418878), seeds_number: 4 },
        SearchResult { title: "Robert Miles - Dreamland - 1996, ALAC, lossless".into(), topic_id: TopicId(1201152), seeds_number: 3 },
        SearchResult { title: "(Trance) Robert Miles - Dreamland (Remastered) - 2016, MP3, 320 kbps".into(), topic_id: TopicId(5309922), seeds_number: 9 },
        SearchResult { title: "(Dream House) Robert Miles - Dreamland (Including One and One) - 1996 [WEB], AAC (tracks) 256 kbps".into(), topic_id: TopicId(4737164), seeds_number: 2 },
    ];

    assert_eq!(7, results.len());
    assert_eq!(expected_results, results);
}

#[test]
fn test_parsing_of_topic() {
    let parsed_topic = parse_topic(include_str!("fixtures/topic.html"))
        .expect("Expected successful parse results");

    assert_eq!(
        Some(Topic {
            topic_id: TopicId(5309922),
            download_id: DownloadId(5309922)
        }),
        parsed_topic
    );
}

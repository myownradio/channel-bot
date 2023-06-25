pub(crate) fn contains_ignore_case(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

pub(crate) fn contains_in_filename_ignore_case(filepath: &str, needle: &str) -> bool {
    match filepath.split(std::path::MAIN_SEPARATOR_STR).last() {
        Some(filename) => contains_ignore_case(&filename, needle),
        None => false,
    }
}

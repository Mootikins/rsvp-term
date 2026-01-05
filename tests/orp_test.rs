use rsvp_term::orp::calculate_orp;

#[test]
fn test_orp_short_words() {
    assert_eq!(calculate_orp("a"), 0); // 1 char
    assert_eq!(calculate_orp("to"), 0); // 2 chars
    assert_eq!(calculate_orp("the"), 0); // 3 chars
}

#[test]
fn test_orp_medium_words() {
    assert_eq!(calculate_orp("word"), 1); // 4 chars
    assert_eq!(calculate_orp("quick"), 1); // 5 chars
    assert_eq!(calculate_orp("jumped"), 1); // 6 chars
}

#[test]
fn test_orp_longer_words() {
    assert_eq!(calculate_orp("quickly"), 2); // 7 chars
    assert_eq!(calculate_orp("beautiful"), 2); // 9 chars
}

#[test]
fn test_orp_very_long_words() {
    assert_eq!(calculate_orp("extraordinary"), 3); // 13 chars -> 10+
    assert_eq!(calculate_orp("unbelievable"), 3); // 12 chars -> 10+
}

#[test]
fn test_orp_unicode_words() {
    // "ということ" is 5 Unicode chars, so should return 1 (4-6 range)
    assert_eq!(calculate_orp("ということ"), 1);
}

#[test]
fn test_orp_empty_string() {
    assert_eq!(calculate_orp(""), 0);
}

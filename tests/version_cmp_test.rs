use appcu::version_cmp;

#[test]
fn version_cmp_test() {
    let ordering1 = version_cmp::cmp_version("1.0.0", "1.0.0", false);
    assert_eq!(ordering1, std::cmp::Ordering::Equal);

    let ordering2 = version_cmp::cmp_version("1.0.1", "1.0.0", false);
    assert_eq!(ordering2, std::cmp::Ordering::Greater);

    let ordering3 = version_cmp::cmp_version("1.0.1", "1.0", false);
    assert_eq!(ordering3, std::cmp::Ordering::Equal);

    let ordering4 = version_cmp::cmp_version("1.0.1", "1.0", true);
    assert_eq!(ordering4, std::cmp::Ordering::Greater);
}

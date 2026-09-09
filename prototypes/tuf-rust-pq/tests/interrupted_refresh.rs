mod interrupted_refresh_fixture;

#[test]
fn observes_one_interrupted_tough_metadata_refresh() {
    interrupted_refresh_fixture::observe().unwrap();
}

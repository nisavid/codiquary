mod interrupted_refresh_fixture;

#[test]
fn observes_one_interrupted_tough_metadata_refresh() {
    interrupted_refresh_fixture::observe().unwrap();
}

#[test]
fn records_completed_loader_non_acceptance() {
    interrupted_refresh_fixture::record_completed_loader_non_acceptance().unwrap();
}

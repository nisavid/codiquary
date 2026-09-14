fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("conformance") => {
            for case in codiquary::cases() {
                println!(
                    "{}\t{}\t{}",
                    case.id,
                    case.expected.as_str(),
                    case.description
                );
            }
        }
        _ => {
            eprintln!("usage: cophax conformance");
            std::process::exit(2);
        }
    }
}

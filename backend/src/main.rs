use config::CONFIG;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match dotenvy::dotenv_override() {
        Ok(_) => {}
        Err(e) if e.not_found() => {
            eprintln!("Couldn't find .env file: {:?}", e);
        }
        Err(e) => {
            println!("Error loading .env file: {:?}", e);
            std::process::exit(1);
        }
    }

    logger::init();
    logger::debug!(config = ?*CONFIG, "loaded config");

    api::run().expect("api::run failed");

    Ok(())
}

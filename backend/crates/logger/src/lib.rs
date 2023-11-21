pub use tracing::{debug, error, info, trace, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "meme_watcher=info,db=info,config=info,warn"
                .parse()
                .unwrap()
        }))
        .with(fmt::layer())
        .try_init()
        .expect("setting default subscriber failed");
}

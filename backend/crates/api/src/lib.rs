use std::{process::exit, sync::Arc, time::Duration};

use config::CONFIG;
use file_watcher::FileWatcher;
use rocket::{
    shield::{self, Shield},
    Route,
};
use tokio::{task, time};

#[macro_use]
extern crate rocket;

mod fairings;
mod helpers;
mod routes;
mod setup;

#[derive(Clone, Debug)]
pub struct AppRoutes(pub Vec<Route>);

#[rocket::main]
pub async fn run() -> Result<(), rocket::Error> {
    let rocket = rocket::build();
    let config = rocket::Config {
        port: CONFIG.server.port,
        address: CONFIG.server.host.parse().unwrap_or_else(|e| {
            logger::error!("failed to parse server address: {}", e);
            exit(1);
        }),
        ident: rocket::config::Ident::none(),
        secret_key: rocket::config::SecretKey::derive_from(CONFIG.server.secret_key.as_bytes()),
        log_level: rocket::config::LogLevel::Off,
        shutdown: rocket::config::Shutdown {
            ctrlc: false,
            ..Default::default()
        },
        ..Default::default()
    };

    logger::debug!(config = ?*CONFIG, "loaded config");

    let db = match setup::setup_db().await {
        Ok(db) => db,
        Err(e) => {
            logger::error!("failed to setup db: {}", e);
            return Ok(());
        }
    };
    let db = Arc::new(db);

    let fw = Arc::new(FileWatcher::new(db.clone()).with_recursive(false));

    let mut instance = rocket
        .configure(config)
        .manage(db.clone())
        .manage(fw.clone())
        .attach(
            Shield::default()
                .enable(shield::Prefetch::Off)
                .enable(shield::Referrer::StrictOriginWhenCrossOrigin),
        )
        .attach(fairings::AddRequestId)
        .attach(fairings::RequestLogger::default());

    for (base, routes) in routes::get() {
        instance = instance.mount(base, routes);
    }

    let routes = instance.routes().cloned().collect();
    let instance = instance.manage(AppRoutes(routes));

    let instance = instance.ignite().await?;

    task::spawn(async move {
        loop {
            logger::info!("starting file inspection");
            let res = fw.index_files().await;
            if let Err(e) = res {
                logger::error!("failed to inspect files: {}", e);
            } else {
                logger::info!("finished file inspection");
            }
            time::sleep(Duration::from_secs(60)).await;
        }
    });

    instance.launch().await?;

    Ok(())
}

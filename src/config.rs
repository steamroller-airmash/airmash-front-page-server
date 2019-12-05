use crate::spec::*;
use crate::CONFIG_FILE;

use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use std::time::Duration;
use std::sync::Arc;

use arc_swap::ArcSwap;

const CONFIG_PATH: &str =
	r"https://raw.githubusercontent.com/airmash-refugees/airmash-games/master/games.txt";
const REGIONS_PATH: &str = 
    r"https://raw.githubusercontent.com/airmash-refugees/airmash-games/master/regions.txt";

fn fetch(path: &str) -> Option<String> {
    let resp = attohttpc::get(path).send().ok()?;

	if !resp.is_success() {
        warn!("Failed to fetch response from {}", path);
		return None;
	}

	resp.text().ok()
}

fn parse_config(config: &str, regions: &str) -> Option<GameSpec> {
    let mut lookup = HashMap::<_, _, RandomState>::default();

    for line in regions.split("\n") {
        // Skip the line at the end
        if line == "" {
            continue;
        }

        let mut fields = line.split("|");

        let id = fields.next()?;
        let name = fields.next()?;

        lookup.insert(id, name);
    }

	let mut regions = HashMap::<_, _, RandomState>::default();

	for line in config.split("\n") {
        // Skip the final line
        if line == "" {
            continue;
        }

		let mut fields = line.split("|");

		let region: &str = fields.next()?;
		let ty = fields.next()?.parse().ok()?;
		let id = fields.next()?.to_string();
		let name = fields.next()?.to_string();
		let short = fields.next()?.to_string();
		let host = fields.next()?.to_string();
		let path = fields.next()?.to_string();

		let url = "wss://".to_string() + &host + "/" + &path;

		let spec = ServerSpec {
			url,
			ty,
			id,
			name,
			name_short: short,
			host,
			path,
			players: None,
        };
        
        regions.entry(region)
            .or_insert(vec![])
            .push(spec);
    }
    
    let mut spec = GameSpec {
        protocol: 8,
        country: "".to_string(),
        data: Vec::new()
    };

    for (region, servers) in regions.into_iter() {
        let region = RegionSpec {
            name: lookup.get(region)?.to_string(),
            id: region.to_string(),
            games: servers
        };

        spec.data.push(region);
    }

	Some(spec)
}

fn fetch_config() -> Option<GameSpec> {
    let config = fetch(CONFIG_PATH)?;
    let regions = fetch(REGIONS_PATH)?;

    parse_config(&config, &regions)
}

lazy_static! {
    static ref CONFIG: ArcSwap<GameSpec> = ArcSwap::new(Arc::new(serde_json::from_str(CONFIG_FILE).unwrap()));
}

pub(crate) fn background_update() {
    loop {
        if let Some(config) = fetch_config() {
            CONFIG.store(Arc::new(config));
            info!("Reloaded config!");
        } else {
            warn!("Failed to load updated config!");
        }

        // Update once every hour
        std::thread::sleep(Duration::from_secs(60 * 60));
    }
}

pub(crate) fn get_config() -> Arc<GameSpec> {
    CONFIG.load_full()
}

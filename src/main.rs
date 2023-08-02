#[macro_use] extern crate rocket;
use base64::Engine;
use reqwest::blocking::Client;
use rocket::{State, serde};
use rocket::fs::NamedFile;
use rocket::fs::FileServer;
use rocket::response::stream::{Event, EventStream};
use rocket::tokio::sync::watch::{channel, Sender};

use reqwest::header;

use std::collections::HashMap;
use std::time::Duration;
use std::env;

mod json_types;
mod types;

use types::*;
use json_types::*;

fn load_config(path: &str) -> Config {
    let raw = std::fs::read_to_string(path).unwrap();
    return serde::json::from_str::<Config>(&raw).unwrap();
}

fn build_client() -> Client {
    let auth = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", env::var("KIDZONE_APP_ID").unwrap(), env::var("KIDZONE_SECRET_KEY").unwrap()));
    let mut headers = header::HeaderMap::new();

    let mut auth_value = header::HeaderValue::from_str(&format!("Basic {}", auth)).unwrap();
    auth_value.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, auth_value);

    Client::builder().default_headers(headers).build().unwrap()
}

fn poll(tx: &Sender<HashMap<String, Location>>, client: &Client, evt_id: &String, locations: &[String]) {
    tx.send_if_modified(|data| {
        let mut changed = false;
        for levt_id in locations {
            let count = fetch_location_event_time(client, &evt_id, levt_id).unwrap();
            let loc = data.get_mut(levt_id).unwrap();
            if loc.count != count {
                changed = true;
                loc.count = count;
            }
        }
        changed
    });
}

fn fetch_current_event_times(client: &Client, id: &str) -> Result<String, ()> {
    let url = format!("https://api.planningcenteronline.com/check-ins/v2/events/{}/current_event_times", id);
    let response = client.get(url).send().unwrap();
    let parsed = response.json::<JsonCurrentEventTimes>().unwrap();

    Ok(parsed.data[0].id.clone())
}

fn fetch_all_location_event_times(client: &Client, evt_id: &str) -> Result<HashMap<String, (String, u64)>, ()> {
    let url = format!("https://api.planningcenteronline.com/check-ins/v2/event_times/{}/location_event_times?include=location", evt_id);
    let response = client.get(url).send().unwrap();
    let parsed = response.json::<JsonAllLocEventTime>().unwrap();

    let mut map = HashMap::with_capacity(parsed.data.len());

    for item in parsed.data {
        let count = item.attributes.guest_count.unwrap_or_default()
            + item.attributes.regular_count.unwrap_or_default();
        map.insert(
            item.relationships.unwrap().location.data.id, 
            (item.id, count)
        );
    }

    Ok(map)
}

fn fetch_location_event_time(client: &Client, evt_id: &str, levt_id: &str) -> Result<u64, ()> {
    let url = format!("https://api.planningcenteronline.com/check-ins/v2/event_times/{}/location_event_times/{}", evt_id, levt_id);
    let response = client.get(url).send().unwrap();
    let parsed = response.json::<JsonLocEventTime>().unwrap();

    let count = parsed.data.attributes.guest_count.unwrap_or_default() + parsed.data.attributes.regular_count.unwrap_or_default();

    Ok(count)
}

fn fetch_location(client: &Client, ev_id: &str, loc_id: &str) -> Result<(String, String, u64), ()> {
    let url = format!("https://api.planningcenteronline.com/check-ins/v2/events/{}/locations/{}", ev_id, loc_id);
    let response = client.get(url).send().unwrap();
    let parsed = response.json::<JsonLocation>().unwrap();

    Ok((parsed.data.id, parsed.data.attributes.name, parsed.data.attributes.max_occupancy))
}

#[get("/")]
async fn index() -> Result<NamedFile, std::io::Error> {
    NamedFile::open("static/index.html").await
}

#[get("/listen")]
fn listen(ctxt: &State<Context>) -> EventStream![Event + '_] {
    let mut rx = (*ctxt.inner()).get_reciever();
    EventStream! {
        let data: Vec<Location>;
        {
            let rec = rx.borrow();
            data = (*rec).values().cloned().collect();
        }
        yield Event::json(&data);

        loop {
            rx.changed().await.unwrap();
            let data: Vec<Location>;
            {
                let rec = rx.borrow();
                data = (*rec).values().cloned().collect();
            }
            yield Event::json(&data)
        }
    }
}

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().unwrap();
    
    let exe_path = env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();

    let config_path = env::var("KIDZONE_CONFIG").unwrap_or(
        String::from(exe_dir.join("config.json").to_str().unwrap())
    );

    let static_path = env::var("KIDZONE_STATIC").unwrap_or(
        String::from(exe_dir.join("static").to_str().unwrap())
    );

    let config = load_config(&config_path);
    let client = build_client();
    let event_time = fetch_current_event_times(&client, &config.event_id).unwrap();
    let mut locations = HashMap::<String, Location>::with_capacity(config.locations.len());
    let mut location_event_times = Vec::<String>::with_capacity(config.locations.len());

    {
        let mut all_levts = fetch_all_location_event_times(&client, &event_time).unwrap();
    
        for (id, lc) in config.locations {
            let data = fetch_location(&client, &config.event_id, &id).unwrap();

            let levt = all_levts.remove(&data.0).unwrap();
            location_event_times.push(levt.0.clone());
            locations.insert(levt.0,
                Location {
                    id: data.0,
                    name: lc.preffered_name.unwrap_or(data.1),
                    capacity: data.2,
                    count: levt.1,
                    order: lc.order,
                }
            );
        }
    }

    let (tx, rx) = channel(locations);
    let context = Context::new(rx);

    // TODO: have Tokio or Rocket somehow manage this to prevent a panic on shutdown
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(5));
            poll(&tx, &client, &event_time, &location_event_times)
        }
    });

    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(3));
        open::that("http://localhost:8000").unwrap()
    });

    rocket::build()
        .manage(context)
        .mount("/", routes![index, listen])
        .mount("/static", FileServer::from(static_path))
}

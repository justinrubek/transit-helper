use std::path::{PathBuf, Path};
use clap::Parser;
use gtfs_rt::{FeedEntity, Position};
use prost::Message;
use serde::{Deserialize, Serialize};
use tokio::{spawn, fs::File, io::AsyncWriteExt};
use tokio_schedule::{every, Job};

mod cli;

use cli::Cli;

#[derive(Debug, Deserialize, Serialize)]
struct PositionData {
    latitude: f32,
    longitude: f32,
}

#[derive(Debug, Deserialize, Serialize)]
struct VehicleData {
    route: String,
    position: PositionData,
}

fn get_dart_route(feed: FeedEntity) -> Option<VehicleData> {
    let vehicle = if let Some(vehicle) = feed.vehicle {
        vehicle
    } else {
        return None;
    };

    let vehicle_trip = if let Some(trip) = vehicle.trip {
        trip
    } else {
        return None;
    };

    let route_id = if let Some(route_id) = vehicle_trip.route_id {
        route_id
    } else {
        return None;
    };

    let position = if let Some(position) = vehicle.position {
        position
    } else {
        return None;
    };

    Some(VehicleData {
        route: route_id,
        position: PositionData {
            latitude: position.latitude,
            longitude: position.longitude,
        },
    })
}

struct PositionLogger {
    client: reqwest::Client,
}

impl PositionLogger {
    fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    async fn log_position_data(&self, path: &str) {
        // Fetch the data immediately once
        self.retrieve_data(path).await;

        let job = every(30)
            .seconds()
            .perform(|| self.retrieve_data(path));

        job.await;
    }

    async fn retrieve_data(&self, path: &str) {
        let position_resp = self.client.get("https://www.ridedart.com/gtfs/real-time/vehicle-positions").send().await.unwrap();

        let data = position_resp.bytes().await.unwrap();
        let message = gtfs_rt::FeedMessage::decode(data).unwrap();

        // convert timestamp to iso8601
        let timestamp: i64 = message.header.timestamp.unwrap().try_into().unwrap();
        let datetime = chrono::DateTime::<chrono::Utc>::from_utc(
            chrono::NaiveDateTime::from_timestamp(timestamp, 0),
            chrono::Utc,
        );
        let file_name = format!("{}.json", datetime.format("%Y-%m-%dT%H:%M:%S"));

        let file_path = Path::new(path).join(file_name);

        println!("Writing to file: {}", file_path.display());
        let mut file = File::create(file_path).await.unwrap();
        let route_data = message
            .entity
            .into_iter()
            .filter_map(get_dart_route)
            .collect::<Vec<_>>();

        let json = serde_json::to_string(&route_data).unwrap();
        file.write_all(json.as_bytes()).await.unwrap();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(cli::Commands::LogPositionData { path }) => {
            let logger = PositionLogger::new();
            logger.log_position_data(&path).await;
        }
        Some(cli::Commands::PrintPositionData) => {
            // retrieve and display the current position data
            let resp = reqwest::get("https://www.ridedart.com/gtfs/real-time/vehicle-positions").await?;
            let data = resp.bytes().await?;
            let message = gtfs_rt::FeedMessage::decode(data).unwrap();

            let route_data = message
                .entity
                .into_iter()
                .filter_map(get_dart_route)
                .collect::<Vec<_>>();

            println!("{:#?}", route_data);
        }
        _ => unimplemented!(),
    };

    Ok(())
}

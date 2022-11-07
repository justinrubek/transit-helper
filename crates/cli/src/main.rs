use clap::Parser;
use gtfs_rt::{FeedEntity, Position};
use prost::Message;
use tokio::spawn;
use tokio_schedule::{every, Job};

mod cli;

use cli::Cli;

#[derive(Debug)]
struct VehicleData {
    route: String,
    position: Position,
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

    Some(VehicleData {
        route: route_id,
        position: vehicle.position.unwrap(),
    })
}

async fn log_position_data(path: String) {
    // Fetch the data immediately once
    retrieve_data().await;

    let job = every(30)
        .seconds()
        .perform(retrieve_data);

    job.await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(cli::Commands::LogPositionData { path }) => {
            log_position_data(path.to_str().unwrap().to_string()).await;
        }
        _ => unimplemented!(),
    };

    Ok(())
}

async fn retrieve_data() {
    let client = reqwest::Client::new();

    let position_resp = client.get("https://www.ridedart.com/gtfs/real-time/vehicle-positions").send().await.unwrap();

    let data = position_resp.bytes().await.unwrap();
    let message = gtfs_rt::FeedMessage::decode(data).unwrap();
    // println!("{:?}", message);

    // print message.header info
    println!("timestamp: {:?}", message.header.timestamp);

    println!("entity count: {}", message.entity.len());
    message.entity.iter().for_each(|entity| {
        if let Some(route_data) = get_dart_route(entity.clone()) {
            println!("route {} at {:?}", route_data.route, route_data.position);
        }
    });
}

use gtfs_rt::{FeedEntity, Position};
use prost::Message;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let position_resp = client.get("https://www.ridedart.com/gtfs/real-time/vehicle-positions").send().await?;

    let data = position_resp.bytes().await?;
    let message = gtfs_rt::FeedMessage::decode(data)?;
    // println!("{:?}", message);

    // print message.header info
    println!("timestamp: {:?}", message.header.timestamp);

    println!("entity count: {}", message.entity.len());
    message.entity.iter().for_each(|entity| {
        if let Some(route_data) = get_dart_route(entity.clone()) {
            println!("route {} at {:?}", route_data.route, route_data.position);
        }
    });

    Ok(())
}

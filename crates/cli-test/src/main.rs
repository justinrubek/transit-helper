use prost::Message;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fname = std::env::args().nth(1).expect("missing file name");
    let data = std::fs::read(fname)?;
    let message = gtfs_rt::FeedMessage::decode(&*data)?;
    // println!("{:?}", message);

    // print message.header info
    println!("timestamp: {:?}", message.header.timestamp);

    println!("entity count: {}", message.entity.len());
    for entity in message.entity {
        // Extract the route_id from entity.vehicle.trip.route_id
        // where each field is an Option
        let vehicle = if let Some(vehicle) = entity.vehicle {
            vehicle
        } else {
            continue;
        };

        let vehicle_trip = if let Some(trip) = vehicle.trip {
            trip
        } else {
            continue;
        };

        let route_id = if let Some(route_id) = vehicle_trip.route_id {
            route_id
        } else {
            continue;
        };

        println!(r#"route {:?}
            position: {:?}
        "#,
        route_id,
        vehicle.position,
        );
    }

    Ok(())
}

use astro_core::{calculate_core_chart, set_ephe_path, BirthData};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Point to bundled ephemeris data; change to your ephe directory if needed.
    set_ephe_path("src/swisseph/ephe");

    let birth = BirthData {
        year: 1990,
        month: 7,
        day: 15,
        hour: 10,      // UTC, 24h
        minute: 30,
        second: 0.0,
        lat: 40.7128,  // +N latitude
        lon: -74.0060, // +E longitude, -W for west
    };

    let chart = calculate_core_chart(&birth)?;

    println!("Sun sign: {}", chart.sun_sign);
    println!("Moon sign: {}", chart.moon_sign);
    println!("Ascendant: {}", chart.asc_sign);

    Ok(())
}

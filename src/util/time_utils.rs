use chrono::Local;
use chrono::NaiveTime;
pub fn get_time_axis_data() -> Result<Vec<String>, anyhow::Error> {
    // Get the current local time
    let current_time = Local::now().time();

    // Create a starting time from midnight (00:00)
    let start_time = NaiveTime::from_hms_opt(0, 0, 0).ok_or(anyhow!(""))?;

    // Collect all the times in "hh:mm" format from start_time to current_time
    let mut times = Vec::new();
    let mut time = start_time;
    while time <= current_time {
        times.push(time.format("%H:%M").to_string());
        // Add one minute to the time
        time += chrono::Duration::minutes(1);
    }
    Ok(times)
}

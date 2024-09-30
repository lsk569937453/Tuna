use chrono::Duration;
use chrono::Local;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
pub fn get_time_axis_data() -> Result<Vec<String>, anyhow::Error> {
    let now = Local::now();

    // Get the start of the current day (midnight)
    let today = now.date_naive();
    let start_of_day = NaiveDateTime::new(today, NaiveTime::from_hms(0, 0, 0));

    // Create a list to store the timestamps
    let mut time_list = Vec::new();

    // Iterate from the start of the day, incrementing one second at a time
    let mut current_time = start_of_day;
    while current_time <= now.naive_local() {
        // Format the current time as "YYYY-MM-DD HH:MM:SS" and add to the list
        time_list.push(current_time.format("%Y-%m-%d %H:%M:00").to_string());

        // Increment by 1 second
        current_time += Duration::minutes(1);
    }

    Ok(time_list)
}

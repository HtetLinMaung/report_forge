use regex::Regex;
use std::error::Error;
use tokio_postgres::Client;

pub async fn process_template(
    html_template: &str,
    db_pool: &Client,
) -> Result<String, Box<dyn Error>> {
    // Adjusted regex pattern to handle new lines and any spaces within the SQL tag
    let re = Regex::new(r"\{\{#sql\(([\s\S]*?)\)\}\}([\s\S]*?)\{\{/sql\}}")?;
    let placeholder_re = Regex::new(r"\{\{(\w+)\}\}")?;

    let mut processed_html = String::from(html_template);

    for cap in re.captures_iter(html_template) {
        let query = cap[1].trim().replace("\n", " "); // Trim whitespace and replace newlines
        let template = &cap[2];

        // Execute the SQL query, handle errors properly
        let rows = db_pool.query(&query, &[]).await?;

        let mut generated_html = String::new();
        for row in rows {
            println!("{:?}", row);
            let mut row_html = String::from(template);

            // Iterate through all placeholders in the template
            // for placeholder_cap in placeholder_re.captures_iter(template) {
            //     let placeholder = &placeholder_cap[0]; // e.g., "{{name}}"
            //     let column_name = &placeholder_cap[1]; // e.g., "name"

            //     if let Ok(value) = row.try_get::<&str, &str>(column_name) {
            //         row_html = row_html.replace(placeholder, value);
            //     } else {
            //         // Insert a default value or handle the error appropriately
            //         row_html = row_html.replace(placeholder, "");
            //     }
            // }

            for placeholder_cap in placeholder_re.captures_iter(template) {
                let placeholder = &placeholder_cap[0]; // e.g., "{{name}}"
                let column_name = &placeholder_cap[1]; // e.g., "name"

                // Attempt to get the value from the row.
                // Try as string, then as i32, then as f64, etc.
                let value: String = if let Ok(val) = row.try_get::<&str, String>(column_name) {
                    val
                } else if let Ok(val) = row.try_get::<&str, i32>(column_name) {
                    val.to_string()
                } else if let Ok(val) = row.try_get::<&str, f64>(column_name) {
                    val.to_string()
                } else {
                    // If none of the above types match, you might want to log this case or handle it specifically.
                    // For now, we'll just return an empty string.
                    "".to_owned()
                };

                // Replace the placeholder with the actual data or an empty string if the value could not be retrieved.
                row_html = row_html.replace(placeholder, &value);
            }

            generated_html.push_str(&row_html);
        }

        processed_html = processed_html.replacen(&cap[0], &generated_html, 1);
    }

    Ok(processed_html)
}

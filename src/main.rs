use anyhow::{ Context, Error };
use dotenv::dotenv;
use reqwest::Client;
use rusqlite::{ params, Connection, Result as SqliteResult };
use serde::{ Deserialize, Serialize };
use std::env;
use std::time::Duration;
use tokio::time;
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
struct Event {
    eventId: String,
    blockNumber: u64,
    transactionHash: String,
    name: String,
    timestamp: i64,
    #[serde(flatten)]
    extra: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    items: Vec<Event>,
    lastPage: u32,
}

async fn init_db() -> SqliteResult<Connection> {
    let conn = Connection::open("starkcron_voyager.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS events (
            event_id TEXT PRIMARY KEY,
            block_number INTEGER,
            transaction_hash TEXT,
            name TEXT,
            timestamp INTEGER,
            data TEXT
        )",
        []
    )?;
    Ok(conn)
}

fn event_exists(conn: &Connection, event_id: &str) -> SqliteResult<bool> {
    let mut stmt = conn.prepare("SELECT 1 FROM events WHERE event_id = ? LIMIT 1")?;
    let exists = stmt.exists(params![event_id])?;
    Ok(exists)
}

fn store_event(conn: &Connection, event: &Event) -> SqliteResult<()> {
    conn.execute(
        "INSERT OR REPLACE INTO events (event_id, block_number, transaction_hash, name, timestamp, data)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            event.eventId,
            event.blockNumber,
            event.transactionHash,
            event.name,
            event.timestamp,
            serde_json::to_string(&event).unwrap()
        ]
    )?;
    Ok(())
}

async fn poll_voyager_api(
    client: &Client,
    contract: &str,
    api_key: &str,
    page: u32
) -> Result<ApiResponse, Error> {
    let api_url = "https://sepolia-api.voyager.online/beta/events";
    let mut url = Url::parse(api_url)?;
    url.query_pairs_mut()
        .append_pair("ps", "10")
        .append_pair("p", &page.to_string())
        .append_pair("contract", contract);

    let response = client
        .get(url)
        .header("accept", "application/json")
        .header("x-api-key", api_key)
        .send().await?
        .json::<ApiResponse>().await?;
    Ok(response)
}

async fn call_starklens_api(client: &Client, events: &[Event]) -> Result<(), Error> {
    let api_url = "https://starklens.vercel.app/api/indexer";
    // let api_url = "http://localhost:3000/api/indexer";
    client
        .post(api_url)
        .json(&serde_json::json!({ "items": events }))
        .send().await?;
    println!("Successfully sent {} events to Starklens API", events.len());
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    let contract = env
        ::var("STARKLENS_SWAPERC20_CONTRACT")
        .context("STARKLENS_SWAPERC20_CONTRACT must be set")?;
    let api_key = env::var("VOYAGER_SECRET").context("VOYAGER_SECRET must be set")?;

    let conn = init_db().await?;
    let client = Client::new();

    loop {
        println!("Polling Voyager API...");
        let mut page = 1;
        let mut all_new_events = Vec::new();

        loop {
            let response = poll_voyager_api(&client, &contract, &api_key, page).await?;
            let events = response.items;

            let mut new_events = Vec::new();
            for event in events {
                if !event_exists(&conn, &event.eventId)? {
                    store_event(&conn, &event)?;
                    new_events.push(event);
                }
            }

            all_new_events.extend(new_events);

            if page >= response.lastPage {
                break;
            }
            page += 1;
        }

        // Reverse the vector
        all_new_events.reverse();

        if !all_new_events.is_empty() {
            call_starklens_api(&client, &all_new_events).await?;
        }

        time::sleep(Duration::from_secs(20)).await;
    }
}

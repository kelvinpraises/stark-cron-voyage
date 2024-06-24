# StarkCron Voyager

StarkCron Voyager is a Rust-based cron job that bridges the Voyager API and Starklens API for StarkNet event tracking. It periodically fetches new events from the Voyager API, stores them locally, and forwards them to the Starklens API.

## Features

- Polls the Voyager API at regular intervals
- Stores events in a local SQLite database
- Forwards new events to the Starklens API
- Handles API pagination
- Environment-based configuration

## Installation

### For Users

If you just want to use StarkCron Voyager, you can install it globally using npm:

```
npm install -g starkcron-voyager
```

### For Developers

If you want to contribute or modify the project, follow these steps:

1. Prerequisites:
   - Rust (latest stable version)
   - SQLite
   - Node.js and npm

2. Clone the repository:
   ```
   git clone https://github.com/starklens/starkcron-voyager.git
   cd starkcron-voyager
   ```

3. Build the project:
   ```
   cargo build --release
   ```

## Configuration

Create a `.env` file in the directory where you'll run the cron job with the following content:

```
CONTRACT_ADDRESS=your_contract_address
VOYAGER_API_KEY=your_voyager_api_key
```

Replace `your_contract_address` and `your_voyager_api_key` with your actual values.

## Usage

### For Users

After installation, you can run the cron job using:

```
starkcron-voyager
```

### For Developers

After building, you can run the cron job using:

```
cargo run --release
```

The program will start polling the Voyager API every 60 seconds, store new events in a local SQLite database, and forward them to the Starklens API.

## Database

The program uses an SQLite database named `starkcron_voyager.db` to store events. The database is created automatically in the current working directory when the program runs for the first time.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
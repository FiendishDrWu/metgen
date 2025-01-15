# METGen - METAR Generator for Flight Simulators

METGen is a Rust-based utility that generates synthetic METARs (Meteorological Terminal Air Reports) for home flight simulator use. It's specifically designed to provide accurately formatted METARs for airfields and airports that don't have real-world METAR reporting capabilities, or for other reasons have no METAR data available. From tiny barely-known dirt strips, to historical airport locations, or an empty stretch of beach at a scenic location, to an airport that doesn't exist, or even to the airports of the nation of Ukraine. You can create a METAR for anywhere in the world.

## Features

- Generates METARs formatted to North American or European standards
- Uses real-time weather data from various weather APIs
- Converts airport locations to lat/long coordinates using NOAA APIs
- Produces accurate METAR strings based on JSON weather data
- Custom GUI interface
- Supports both standard OpenWeather API and OneCall API
- Includes a comprehensive airport database to fill holes in NOAA data
- Handles multiple input methods: ICAO codes, coordinates, or location names

## Prerequisites

- OpenWeather API key (free tier works)
- Optional: OpenWeather OneCall API subscription for enhanced features

## Getting API Keys

### Standard API Key (Free)
1. Go to [OpenWeather](https://openweathermap.org) and create a free account
2. After signing in, go to your profile and select "My API Keys"
3. Copy your API key or generate a new one
4. The free tier allows up to 60 calls per minute, 1,000,000 calls per month, which is more than sufficient for personal use
5. A free account only requires an email address to register

### OneCall API (Optional Subscription)
1. The OneCall API 3.0 provides enhanced weather data and forecasting
2. Subscribe through [OpenWeather One Call API 3.0](https://openweathermap.org/api/one-call-3)
3. Subscription service requires a credit card and personal information, but can be used for free if setup properly
4. The OneCall API 3.0 subscription is pay as you go, with 1000 calls per day for free
5. The OpenWeather API dashboard allows you to limit your number of OneCall API calls per day, setting your limit to 1000 calls per day will allow you to use the OneCall API for free. Calls in excess of 1000 are charged at $0.0015 each at time of writing
6. You can use the same API key for both services if you have a subscription, or use individual keys for each service

Note: The program works perfectly fine with just the free API key. OneCall features are optional and enhance the METAR generation with additional weather trend data.
Note: The NOAA API is publicly accessible. If you enter an invalid OpenWeather API key, the program will still use the NOAA API to check for an existing METAR and present it to you. Not sure why you'd want to use this strictly to pull actual NOAA METARs, but it's there if you need it.
Note: 

## Installation

### Binary Release (Recommended)
1. Download the executable for your platform (Windows/Linux/macOS) from the [Releases](../../releases) page
2. Place the executable in a folder of your choice
3. You can rename the executable to anything you like, if desired
4. Run the program

### Building From Source (Optional)
If you prefer to build from source, you'll need:
- Rust 2021 edition or later
- Cargo (Rust's package manager)
- For Windows only: A working Windows SDK installation for resource compilation

Steps:
1. Clone this repository
2. Run `cargo build --release`
   - On Windows: The build will automatically compile the icon resource file (requires Windows SDK)
   - On Linux/macOS: The build will proceed normally
3. The executable will be available in `target/release/`

## First Run Setup
1. Launch the program
2. Follow the prompts to enter your API key(s)
3. Choose your preferred units (metric/imperial)

## Usage

The program offers several workflows:

1. **Standard METAR Generation**
   - Uses basic OpenWeather API
   - Suitable for most users
   - Free tier compatible

2. **OneCall METAR Generation**
   - Enhanced accuracy
   - Includes trend information
   - Requires OneCall subscription

Input methods:
1. ICAO Code (e.g., KJFK)
2. Latitude/Longitude coordinates
3. Freeform location name (e.g., "Queens", uses the free OpenWeather Geocoding API)

## Configuration

- Config file is automatically created on first run
- API keys are stored encrypted
- Units can be changed anytime

## License

This project is licensed under the GNU Affero General Public License v3 (AGPLv3). This means:

- You can freely use and modify this software
- If you distribute modified versions, you must:
  - Make your source code available
  - License it under AGPLv3
  - Document your changes
- If you use this software to provide a service over a network (e.g., as a web service), you must:
  - Make the complete source code available to users
  - Include all modifications you've made
  - License everything under AGPLv3
- Commercial use must comply with all AGPLv3 requirements

## Important Notice

This software is for simulator use only. It must not be used for actual aviation purposes. 

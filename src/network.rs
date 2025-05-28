// netwoek.rs
use anyhow::Result;
// use std::collections::HashMap;
use std::error::Error;
use std::fmt;
// use std::sync::{Arc, Mutex};
use url::Url;

#[derive(Debug)]
pub struct BrowserError(String);

impl fmt::Display for BrowserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for BrowserError {}

pub fn load_url(url_str: &str) -> Result<String> {
    println!("URL: {}", url_str);

    let parsed_url = Url::parse(url_str)?;

    let scheme = parsed_url.scheme().to_string();
    // only suport http & https
    if scheme != "http" && scheme != "https" {
        return Err(BrowserError(format!("Unsupported scheme: {}", scheme)).into());
    }

    let body = reqwest::blocking::get(url_str)?.text()?;

    Ok(body)
}

// Manages network request's for browsers & caches files
// pub struct NetworkManager {
//     cache_audio: Mutex<HashMap<String, Arc<Vec<u8>>>>,
// }

// impl NetworkManager {
//     pub fn new() -> Self {
//         Self {
//             cache_audio: Mutex::new(HashMap::new()),
//         }
//     }

//     // Gets audio data from cache or network. Returns None if error occurs.
//     pub fn get_audio_data(&self, url: &String) -> Option<Arc<Vec<u8>>> {
//         // Acquire the mutex lock to check the cache.
//         // Return None if locking fails (poisoned mutex).
//         let cache = self.cache_audio.lock().ok()?;

//         // Check if the data is already in the cache.
//         if let Some(aud) = cache.get(url) {
//             // Found in cache, return clone and release the lock.
//             return Some(aud.clone());
//         }

//         // Data not in cache. Release the lock before making the network request.
//         // This prevents blocking other threads waiting for the cache lock
//         // while we perform blocking I/O.
//         drop(cache);

//         // Fetch data from the network. Return None if the request fails.
//         let response = reqwest::blocking::get(url).ok()?;

//         // Read the response body as bytes. Return None if reading fails.
//         let bytes = response.bytes().ok()?.to_vec();

//         // Re-acquire the mutex lock to insert into the cache.
//         let mut cache = self.cache_audio.lock().ok()?;

//         // IMPORTANT: Re-check the cache *after* acquiring the lock again.
//         // Another thread might have fetched and inserted the data while we were
//         // performing the network request for the same URL.
//         if let Some(aud) = cache.get(url) {
//             // Another thread inserted it while we were busy. Return their data.
//             // The bytes we fetched will be dropped.
//             return Some(aud.clone());
//         }

//         // Data is still not in cache (or was removed/replaced, which is unlikely here).
//         // Insert the data we just fetched.
//         let audio_data_arc = Arc::new(bytes);
//         cache.insert(url.clone(), audio_data_arc.clone());

//         // Return the data we just inserted.
//         Some(audio_data_arc)
//     }
// }

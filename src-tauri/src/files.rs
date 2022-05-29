use serde::{
    de::DeserializeOwned,
    Serialize
};

use std::{
    fs,
    io::{
        self,
        BufReader
    },
    path::PathBuf
};

pub async fn save_file<T>(filename: &String, partners: &T) -> Result<(), io::Error>
where T: Serialize {
    let serialized = serde_json::to_string_pretty::<T>(partners)?;
    tokio::fs::write(PathBuf::from(filename), serialized).await
}

pub fn get_object_or_default<T: Default + DeserializeOwned>(filename: &String) -> T {
    match read_file(filename) {
        Ok(data) => data,
        Err(err) => {
            if err.kind() != io::ErrorKind::NotFound {
                error!("failed to read the file: {}", err);
            }
            info!("file {} not found, using default", filename);
            T::default()
        }
    }
}

fn read_file<T: DeserializeOwned>(filename: &String) -> Result<T, io::Error> {
    let buf_reader = BufReader::new(fs::File::open(filename)?);
    let data: T = serde_json::from_reader(buf_reader)?;

    Ok(data)
}

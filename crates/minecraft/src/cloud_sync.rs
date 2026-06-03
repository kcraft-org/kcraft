use reqwest::blocking::Client;
use std::fs::{self, File};
use std::io;
use std::path::Path;
use zip::write::SimpleFileOptions;

pub struct CloudSync {
    client: Client,
    endpoint: String,
}

impl CloudSync {
    pub fn new(endpoint: String) -> Self {
        Self {
            client: Client::new(),
            endpoint,
        }
    }

    fn zip_dir<T>(
        &self,
        it: &mut dyn Iterator<Item = walkdir::DirEntry>,
        prefix: &str,
        writer: T,
    ) -> zip::result::ZipResult<()>
    where
        T: io::Write + io::Seek,
    {
        let mut zip = zip::ZipWriter::new(writer);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for entry in it {
            let path = entry.path();
            let name = path.strip_prefix(Path::new(prefix)).unwrap();

            if path.is_file() {
                #[allow(deprecated)]
                zip.start_file(name.to_str().unwrap(), options)?;
                let mut f = File::open(path)?;
                io::copy(&mut f, &mut zip)?;
            }
        }
        zip.finish()?;
        Ok(())
    }

    pub fn share_instance(
        &self,
        instance_path: &Path,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let metadata = fs::metadata(instance_path)?;
        if !metadata.is_dir() {
            return Err("Instance path must be a directory".into());
        }

        let zip_path = std::env::temp_dir().join("instance_share.zip");
        let file = File::create(&zip_path)?;
        let walkdir = walkdir::WalkDir::new(instance_path);
        let mut it = walkdir.into_iter().filter_map(|e| e.ok());
        self.zip_dir(&mut it, instance_path.to_str().unwrap(), file)?;

        let file_content = fs::read(&zip_path)?;

        let response = self
            .client
            .post(format!("{}/api/share", self.endpoint))
            .body(file_content)
            .send()?;

        let share_url = response.text()?;
        Ok(share_url)
    }

    pub fn import_instance(
        &self,
        url: &str,
        dest_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let response = self.client.get(url).send()?;
        let bytes = response.bytes()?;

        fs::create_dir_all(dest_path)?;
        let zip_path = dest_path.join("instance.zip");
        fs::write(&zip_path, bytes)?;

        let file = File::open(&zip_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => dest_path.join(path),
                None => continue,
            };

            if (*file.name()).ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = fs::File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(())
    }
}

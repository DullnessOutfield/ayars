use ayars::*;

pub fn get_probes() {
    let basepath = get_base_path();

    match walkdir::WalkDir::new(basepath)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("kismet"))
    {
        iter => {
            for entry in iter {
                let path = entry.path().to_str().unwrap_or_default();
                println!("Processing: {}", path);
                
                match get_stas(path) {
                    Ok(devices) => {
                        for device in devices {
                            let probes: Vec<_> = device.probed_ssids();
                            for probe in probes {
                                println!("{probe}")
                            }
                        }
                    }
                    Err(e) => eprintln!("Error processing {}: {}", path, e),
                }
            }
        }
    }
}
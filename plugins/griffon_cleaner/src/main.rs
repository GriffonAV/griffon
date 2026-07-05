use griffon_cleaner::{execute_cleaner_metrics_payload, execute_cleaner_payload};

fn has_flag(flag: &str) -> bool {
    std::env::args().skip(1).any(|arg| arg == flag)
}

fn parse_arg(flag: &str) -> Option<String> {
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        if arg == flag {
            return args.next();
        }
    }

    None
}

fn main() {
    let metrics_only = has_flag("--metrics-only");
    let default_output = if metrics_only {
        "griffon_cleaner_metrics.json"
    } else {
        "griffon_cleaner_report.json"
    };

    let output_path = parse_arg("--output").unwrap_or_else(|| default_output.to_string());

    if metrics_only {
        match execute_cleaner_metrics_payload() {
            Ok(metrics) => match serde_json::to_string_pretty(&metrics) {
                Ok(json_str) => {
                    std::fs::write(&output_path, json_str)
                        .expect("Failed to write metrics to file");
                    println!("Metrics exportées dans {}", output_path);
                }
                Err(e) => {
                    eprintln!("Erreur lors de la sérialisation des metrics : {:?}", e);
                    std::process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("Erreur lors de l'exécution du cleaner : {:?}", e);
                std::process::exit(1);
            }
        }

        return;
    }

    match execute_cleaner_payload() {
        Ok(payload) => match serde_json::to_string_pretty(&payload) {
            Ok(json_str) => {
                std::fs::write(&output_path, json_str).expect("Failed to write report to file");
                println!("Report exporté dans {}", output_path);
            }
            Err(e) => {
                eprintln!("Erreur lors de la sérialisation du rapport : {:?}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Erreur lors de l'exécution du cleaner : {:?}", e);
            std::process::exit(1);
        }
    }
}

use griffon_cleaner::execute_cleaner_payload;

fn main() {
    match execute_cleaner_payload() {
        Ok(payload) => match serde_json::to_string_pretty(&payload) {
            Ok(json_str) => {
                let output_path = std::env::args()
                    .skip(1)
                    .collect::<Vec<_>>()
                    .windows(2)
                    .find(|w| w[0] == "--output")
                    .map(|w| w[1].clone())
                    .unwrap_or_else(|| "griffon_cleaner_report.json".to_string());

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
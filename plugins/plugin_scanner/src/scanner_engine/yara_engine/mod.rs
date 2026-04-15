use yara_x::{Compiler, Rules};

pub struct YaraEngine {
    rules: Rules,
    rule_count: usize,
}

impl YaraEngine {
    pub fn load_from_dir(rules_dir: &str) -> Result<Self, yara_x::ScanError> {
        let mut compiler = Compiler::new();
        let mut rule_count = 0;

        let entries = std::fs::read_dir(rules_dir).expect("Could not get rules");
        for entry in entries {
            let entry = entry.expect("Could not read rule file");
            let path = entry.path();
            if path.is_file() {
                let source = std::fs::read_to_string(&path).expect("could not read rule file");
                compiler
                    .add_source(source.as_str())
                    .expect("could not add rule file");
                rule_count += 1;
            }
        }
        let rules = compiler.build();
        Ok(YaraEngine { rules, rule_count })
    }
}

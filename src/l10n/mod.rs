
use fluent::{FluentBundle, FluentResource, FluentArgs};
use unic_langid::langid;
use std::fs;

pub struct L10n {
    bundle: FluentBundle<FluentResource>,
}

impl L10n {
    pub fn new(lang: &str) -> Self {
        let langid = lang.parse().unwrap_or(langid!("en-US"));
        let mut bundle = FluentBundle::new(vec![langid]);

        let ftl_path = format!("locales/{}.ftl", lang);
        let content = fs::read_to_string(&ftl_path).unwrap_or_else(|_| {
            eprintln!("Warning: Could not load locale '{}', falling back to English.", lang);
            fs::read_to_string("locales/en.ftl").expect("Failed to load en.ftl")
        });

        let resource = FluentResource::try_new(content).expect("Failed to parse FTL file");
        bundle.add_resource(resource).expect("Failed to add resource to bundle");

        Self { bundle }
    }

    pub fn get(&self, key: &str, args: Option<&FluentArgs>) -> String {
        let msg = self.bundle.get_message(key).expect("Message not found");
        let pattern = msg.value().expect("Message has no value");
        
        let mut errors = vec![];
        let value = self.bundle.format_pattern(pattern, args, &mut errors);
        
        if !errors.is_empty() {
            eprintln!("Warning: Errors formatting message '{}': {:?}", key, errors);
        }
        
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_l10n_load_en() {
        let l10n = L10n::new("en");
        assert_eq!(l10n.get("ok-status", None), "OK");
    }

    #[test]
    fn test_l10n_fallback() {
        let l10n = L10n::new("de"); // de doesn't exist
        assert_eq!(l10n.get("ok-status", None), "OK");
    }
}

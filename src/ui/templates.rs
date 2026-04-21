use minijinja::Environment;

pub fn create_environment() -> Environment<'static> {
    let mut env = Environment::new();
    env.set_loader(|name| {
        let path = std::path::Path::new("templates").join(name);
        Ok(std::fs::read_to_string(path).ok())
    });
    env.add_filter("display_status", display_status);
    env
}

fn display_status(status: String) -> String {
    status
        .replace('_', " ")
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            c.next()
                .map(|f| f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}
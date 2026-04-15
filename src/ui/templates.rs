use minijinja::Environment;

pub fn create_environment() -> Environment<'static> {
    let mut env = Environment::new();
    env.set_loader(|name| {
        let path = std::path::Path::new("templates").join(name);
        Ok(std::fs::read_to_string(path).ok())
    });
    env
}
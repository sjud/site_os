lazy_static::lazy_static! {
    pub static ref SPACES_KEY: String = {
        if let Ok(secret) = std::env::var("SPACES_KEY") {
             secret
        } else {
            #[cfg(feature="local_env")]
            return dotenv_codegen::dotenv!("SPACES_KEY").to_string();
            #[cfg(not(feature="local_cdn"))]
            "".to_string()
        }
    };
}
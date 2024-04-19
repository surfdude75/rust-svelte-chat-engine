use npm_rs::{NodeEnv, NpmEnv};

fn main() {
    NpmEnv::default()
        .with_node_env(&NodeEnv::from_cargo_profile().unwrap_or_default())
        .set_path("web")
        .init_env()
        .install(None)
        .run("build")
        .exec()
        .expect("build web app");
}

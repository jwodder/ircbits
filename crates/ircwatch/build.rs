use anyhow::Context;

fn main() -> anyhow::Result<()> {
    if let Some(work_tree) = githash::get_work_tree()? {
        println!("cargo::rerun-if-changed={work_tree}/.git/HEAD");
        println!("cargo::rerun-if-changed={work_tree}/.git/refs");
        let pkg_version = getenv("CARGO_PKG_VERSION")?;
        let commit = githash::get_commit_hash(&work_tree)?;
        let git_version = githash::add_semver_metadata(&pkg_version, &format!("g{commit}"))?;
        println!("cargo::rustc-env=VERSION_WITH_GIT={git_version}");
    } else {
        println!("cargo::rerun-if-changed=build.rs");
    }
    Ok(())
}

fn getenv(name: &str) -> anyhow::Result<String> {
    std::env::var(name).with_context(|| format!("{name} envvar not set"))
}

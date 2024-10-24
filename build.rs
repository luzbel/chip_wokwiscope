fn main() {
    println!("cargo:rerun-if-changed=wokwi-chip.json");
    std::fs::copy(
        "wokwi-chip.json",
        format!(
            "target/{}/{}/chip_wokwiscope.json",
            std::env::var("TARGET").unwrap(),
            std::env::var("PROFILE").unwrap(),
        ),
    )
    .unwrap();
}

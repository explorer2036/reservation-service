use proto_builder_trait::tonic::BuilderAttributes;
use std::process::Command;

fn main() {
    tonic_build::configure()
        .out_dir("src/pb")
        .with_sqlx_type(&["reservation.ReservationStatus"], None)
        .with_derive_builder(
            &[
                "reservation.ReservationQuery",
                "reservation.ReservationFilter",
            ],
            None,
        )
        .with_type_attributes(
            &[
                "reservation.ReservationQuery",
                "reservation.ReservationFilter",
            ],
            &[r#"#[builder(build_fn(name = "private_build"))]"#],
        )
        .with_field_attributes(
            &["page_size"],
            &["#[builder(setter(into), default = \"10\")]"],
        )
        .with_field_attributes(
            &["cursor"],
            &["#[builder(setter(into, strip_option), default)]"],
        )
        .compile(&["proto/reservation.proto"], &["proto"])
        .unwrap();
    Command::new("cargo").arg("fmt").output().unwrap();
    println!("cargo:rerun-if-changed=proto/reservation.proto");
}

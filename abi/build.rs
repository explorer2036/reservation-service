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
        .compile(&["proto/reservation.proto"], &["proto"])
        .unwrap();
    Command::new("cargo").arg("fmt").output().unwrap();
    println!("cargo:rerun-if-changed=proto/reservation.proto");
}

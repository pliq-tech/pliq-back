fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &[
                "proto/ai_service.proto",
                "proto/fraud_detection.proto",
                "proto/matching.proto",
            ],
            &["proto/"],
        )?;
    Ok(())
}

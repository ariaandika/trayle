
fn main() -> Result<(), Box<dyn std::error::Error>> {
    trayle::backend::winit::run()?;
    // trayle::backend::udev::setup()
    Ok(())
}


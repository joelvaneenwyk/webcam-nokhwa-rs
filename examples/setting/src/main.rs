use nokhwa::{utils::KnownCameraControl, Camera};
use nokhwa::{
    nokhwa_initialize,
    pixel_format::RgbFormat,
    query,
    utils::{ApiBackend, RequestedFormat, RequestedFormatType},
};

fn main() {
    // only needs to be run on OSX
    nokhwa_initialize(|granted| {
        println!("User said {}", granted);
    });
    let cameras = query(ApiBackend::Auto).unwrap();
    cameras.iter().for_each(|cam| println!("{:?}", cam));

    let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
    let first_camera = cameras.first().unwrap().index();

    let mut camera = Camera::new(first_camera.clone(), format).unwrap();
    let known = camera.camera_controls_known_camera_controls().unwrap();
    let control = known.get(&KnownCameraControl::Gamma).unwrap();
    camera.set_camera_control(control.control(), control.value()).unwrap();
}

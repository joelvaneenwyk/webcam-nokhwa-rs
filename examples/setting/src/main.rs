use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, KnownCameraControl, RequestedFormat, RequestedFormatType},
    Camera,
};

fn main() {
    let requested = RequestedFormat::new::<RgbFormat>(
        RequestedFormatType::None);
    let mut camera = Camera::new(CameraIndex::Index(0), requested).unwrap();
    let known = camera.camera_controls_known_camera_controls().unwrap();
    let control = known.get(&KnownCameraControl::Gamma).unwrap();
    //control.set_value(101).unwrap();
    camera.set_camera_control(KnownCameraControl::Gamma, control.value()).unwrap();
}

use smithay::reexports::drm::control::{
    connector,
    Device as ControlDevice
};

use libdisplay_info::info::Info as DisplayInfo;

pub fn for_connectors(device: &impl ControlDevice, connector: connector::Handle) -> Option<DisplayInfo> {
    let props = device.get_properties(connector).ok()?;

    let (info,value) = props.into_iter()
        .filter_map(|(handle,value)|{
            let info = device.get_property(handle).ok()?;
            Some((info,value))
        })
        .find(|(info,_)|info.name().to_str()==Ok("EDID"))?;

    let blob = info.value_type().convert_value(value).as_blob()?;
    let data = device.get_property_blob(blob).ok()?;

    DisplayInfo::parse_edid(&data).ok()
}


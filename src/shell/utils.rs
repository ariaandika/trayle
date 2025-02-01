use super::elements::WindowElement;
use smithay::{
    desktop::{self, space::SpaceElement, Space},
    utils::{Logical, Point, Rectangle, Size},
};

pub fn fixup_positions(space: &mut Space<WindowElement>, pointer_location: Point<f64, Logical>) {
    // fixup outputs
    let mut offset = Point::<i32, Logical>::from((0, 0));
    for output in space.outputs().cloned().collect::<Vec<_>>().into_iter() {
        let size = space
            .output_geometry(&output)
            .map(|geo| geo.size)
            .unwrap_or_else(|| Size::from((0, 0)));
        space.map_output(&output, offset);
        desktop::layer_map_for_output(&output).arrange();
        offset.x += size.w;
    }

    // fixup windows
    let mut orphaned_windows = vec![];
    let outputs = space
        .outputs()
        .flat_map(|o| {
            let geo = space.output_geometry(o)?;
            let map = desktop::layer_map_for_output(o);
            let zone = map.non_exclusive_zone();
            Some(Rectangle::new(geo.loc + zone.loc, zone.size))
        })
        .collect::<Vec<_>>();
    for window in space.elements() {
        let window_location = match space.element_location(window) {
            Some(loc) => loc,
            None => continue,
        };
        let geo_loc = window.bbox().loc + window_location;

        if !outputs.iter().any(|o_geo|o_geo.contains(geo_loc)) {
            orphaned_windows.push(window.clone());
        }
    }
    for window in orphaned_windows {
        self::place_new_window(space, pointer_location, &window, false)
    }
}

fn place_new_window(
    space: &mut Space<WindowElement>,
    pointer_location: Point<f64, Logical>,
    window: &WindowElement,
    activate: bool,
) {
    let output = space
        .output_under(pointer_location)
        .next()
        .or_else(|| space.outputs().next())
        .cloned();
    let output_geometry = output
        .and_then(|o| {
            let geo = space.output_geometry(&o)?;
            let map = desktop::layer_map_for_output(&o);
            let zone = map.non_exclusive_zone();
            Some(Rectangle::new(geo.loc + zone.loc, zone.size))
        })
        .unwrap_or_else(|| Rectangle::from_size((800, 800).into()));

    // set initial toplevel bounds
    if let Some(toplevel) = window.0.toplevel() {
        toplevel.with_pending_state(|state|{
            state.bounds = Some(output_geometry.size);
        });
    }

    let max_x = output_geometry.loc.x + (((output_geometry.size.w as f32) / 3.0) * 2.0) as i32;
    let max_y = output_geometry.loc.y + (((output_geometry.size.h as f32) / 3.0) * 2.0) as i32;

    use rand::distr::{Uniform, Distribution};
    let x_range = Uniform::new(output_geometry.loc.x, max_x);
    let y_range = Uniform::new(output_geometry.loc.y, max_y);

    let (x_range,y_range) = match (x_range,y_range) {
            (Ok(x), Ok(y)) => (x,y),
            _ => {
                tracing::error!("failed to create rand Uniform, further investigation required");
                return;
            }
        };

    let mut rng = rand::rng();
    let x = x_range.sample(&mut rng);
    let y = y_range.sample(&mut rng);

    space.map_element(window.clone(), (x,y), activate);
}



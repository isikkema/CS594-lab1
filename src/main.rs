use std::process::exit;

use clap::{App, Arg};

use rangle::{Model, Rangle, RangleError, RangleMode};
use solid_shader::get_solid_shader;

use crate::{jgraph_display::JgraphDisplay, normal_shader::get_normal_shader};

mod jgraph_display;
mod normal_shader;
mod solid_shader;

fn match_vec3(color: Option<&str>) -> Result<(f32, f32, f32), String> {
    let color = match color {
        Some(v) => v,
        None => return Err("Color not set".to_string()),
    };

    let mut it = color.trim().split_ascii_whitespace();
    let r = it.next();
    let g = it.next();
    let b = it.next();

    let r = match r {
        Some(v) => v,
        None => return Err("Not enough values given".to_string()),
    };

    let g = match g {
        Some(v) => v,
        None => return Err("Not enough values given".to_string()),
    };

    let b = match b {
        Some(v) => v,
        None => return Err("Not enough values given".to_string()),
    };

    let r = match r.parse::<f32>() {
        Ok(v) => v,
        Err(e) => return Err(e.to_string()),
    };

    let g = match g.parse::<f32>() {
        Ok(v) => v,
        Err(e) => return Err(e.to_string()),
    };

    let b = match b.parse::<f32>() {
        Ok(v) => v,
        Err(e) => return Err(e.to_string()),
    };

    match it.next() {
        Some(_) => return Err("Too many values given".to_string()),
        None => (),
    };

    Ok((r, g, b))
}

fn main() -> Result<(), RangleError> {
    let matches = App::new("J-Grangle")
        .arg(
            Arg::with_name("filename")
                .index(1)
                .takes_value(true)
                .value_name("OBJ-FILE")
                .required(true)
                .help("Path to obj file"),
        )
        .arg(
            Arg::with_name("width")
                .index(2)
                .takes_value(true)
                .required(true)
                .help("The width in pixels"),
        )
        .arg(
            Arg::with_name("height")
                .index(3)
                .takes_value(true)
                .required(true)
                .help("The height in pixels"),
        )
        .arg(
            Arg::with_name("background_color")
                .short("b")
                .long("background")
                .takes_value(true)
                .allow_hyphen_values(true)
                .default_value("0 0 0")
                .help("The background color"),
        )
        .arg(
            Arg::with_name("shader")
                .long("shader")
                .takes_value(true)
                .possible_values(&["normal", "solid"])
                .default_value("normal")
                .help("The pre-compiled set of shaders to use"),
        )
        .arg(
            Arg::with_name("shader_color")
                .required_if("shader", "solid")
                .short("c")
                .long("color")
                .takes_value(true)
                .allow_hyphen_values(true)
                .help("The color of the object"),
        )
        .arg(
            Arg::with_name("scale")
                .short("s")
                .long("scale")
                .takes_value(true)
                .allow_hyphen_values(true)
                .default_value("1 1 1")
                .help("The x y z values to scale the model by"),
        )
        .arg(
            Arg::with_name("translate")
                .short("t")
                .long("translate")
                .takes_value(true)
                .allow_hyphen_values(true)
                .default_value("0 0 0")
                .help("The xyz values to translate the model by"),
        )
        .arg(
            Arg::with_name("yaw")
                .long("yaw")
                .takes_value(true)
                .allow_hyphen_values(true)
                .default_value("0")
                .help("The value in radians to rotate the model around the y-axis"),
        )
        .arg(
            Arg::with_name("pitch")
                .long("pitch")
                .takes_value(true)
                .allow_hyphen_values(true)
                .default_value("0")
                .help("The value in radians to rotate the model around the x-axis"),
        )
        .arg(
            Arg::with_name("roll")
                .long("roll")
                .takes_value(true)
                .allow_hyphen_values(true)
                .default_value("0")
                .help("The value in radians to rotate the model around the z-axis"),
        )
        .arg(
            Arg::with_name("mode")
                .short("m")
                .long("mode")
                .takes_value(true)
                .possible_values(&["triangles", "lines", "points"])
                .default_value("triangles")
                .help("The display mode used to render the model"),
        )
        .get_matches();

    let filename = matches.value_of("filename").unwrap();
    let width = matches.value_of("width").unwrap();
    let height = matches.value_of("height").unwrap();
    let background_color = matches.value_of("background_color");
    let shader = matches.value_of("shader").unwrap();
    let color = matches.value_of("shader_color");
    let scale = matches.value_of("scale");
    let translate = matches.value_of("translate");
    let yaw = matches.value_of("yaw").unwrap();
    let pitch = matches.value_of("pitch").unwrap();
    let roll = matches.value_of("roll").unwrap();
    let mode = matches.value_of("mode").unwrap();

    let width = width.parse::<u16>()?;
    let height = height.parse::<u16>()?;
    let background_color = match match_vec3(background_color) {
        Ok(v) => (v.0, v.1, v.2, 1.0),
        Err(e) => {
            eprintln!("{}", e);
            exit(2);
        }
    };
    let scale = match match_vec3(scale) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            exit(2);
        }
    };
    let translate = match match_vec3(translate) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            exit(2);
        }
    };
    let yaw = yaw.parse::<f32>()?;
    let pitch = pitch.parse::<f32>()?;
    let roll = roll.parse::<f32>()?;
    let mode = match mode {
        "triangles" => RangleMode::Triangles,
        "lines" => RangleMode::Lines,
        "points" => RangleMode::Points,
        _ => unreachable!(),
    };

    {
        let display = JgraphDisplay::new(width, height, background_color)?;

        let mut rangle = Rangle::new(Box::new(display))?;
        rangle.set_display_mode(mode);

        let mut model = Model::from_file(filename)?;
        let shader = match shader {
            "normal" => get_normal_shader(width, height, scale, (yaw, pitch, roll), translate, &mut model)?,
            "solid" => {
                let color = match match_vec3(color) {
                    Ok(v) => (v.0, v.1, v.2, 1.0),
                    Err(e) => {
                        eprintln!("{}", e);
                        exit(2);
                    }
                };
                get_solid_shader(width, height, color, scale, (yaw, pitch, roll), translate, &mut model)?
            },
            _ => unreachable!(),
        };

        let _ = rangle.add_model(model, shader.clone());

        rangle.render_scene()?;
    }

    Ok(())
}

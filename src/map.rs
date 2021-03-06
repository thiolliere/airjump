extern crate svgparser;

use self::svgparser::svg;
use self::svgparser::ElementId;
use self::svgparser::AttributeId;
use self::svgparser::svg::ElementEnd;
use physics::{Body, Shape};
use OkOrExit;
use self::svgparser::xmlparser::FromSpan;

pub struct Map {
    pub bodies: Vec<Body>,
    pub start: [f64; 2],
}

const MAP_FILE: &'static str = "map.svg";

enum Error {
    Io(::std::io::Error),
    Svg(svgparser::xmlparser::Error),
    ParseFloat(::std::num::ParseFloatError),
}
impl From<::std::io::Error> for Error {
    fn from(err: ::std::io::Error) -> Error {
        Error::Io(err)
    }
}
impl From<svgparser::xmlparser::Error> for Error {
    fn from(err: svgparser::xmlparser::Error) -> Error {
        Error::Svg(err)
    }
}
impl From<::std::num::ParseFloatError> for Error {
    fn from(err: ::std::num::ParseFloatError) -> Error {
        Error::ParseFloat(err)
    }
}
impl ::std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        use self::Error::*;
        match *self {
            Io(ref e) => write!(fmt, "file `{}`: io error: {}", MAP_FILE, e),
            Svg(ref e) => write!(fmt, "file `{}`: svg parser error: {}", MAP_FILE, e),
            ParseFloat(ref e) => write!(fmt, "file `{}`: svg parser float error: {}", MAP_FILE, e),
        }
    }
}

#[cfg(feature = "exclude_all")]
fn read_map_file() -> Result<String, Error> {
    use std::fs::File;
    use std::io::Read;
    let mut map = String::new();
    File::open(MAP_FILE)?.read_to_string(&mut map)?;
    Ok(map)
}

#[cfg(not(feature = "exclude_all"))]
fn read_map_file() -> Result<&'static str, Error> {
    Ok(include_str!("../map.svg"))
}

fn load_map() -> Result<Map, Error> {
    let text = read_map_file()?;

    let parser = svg::Tokenizer::from_str(&text);

    let mut bodies = Vec::new();

    let mut start = None;

    // bool is whereas it is start and f64 are cx, cy, r
    let mut circle: Option<(bool, Option<f64>,Option<f64>,Option<f64>)> = None;

    // f64 are x, y, width, height
    let mut rect: Option<(Option<f64>,Option<f64>,Option<f64>,Option<f64>)> = None;

    for next in parser {
        match next? {
            svg::Token::ElementStart(svg::QName { local: svg::Name::Svg(ElementId::Circle), .. }) => circle = Some((false, None, None, None)),
            svg::Token::ElementStart(svg::QName { local: svg::Name::Svg(ElementId::Rect), .. }) => rect = Some((None, None, None, None)),
            svg::Token::ElementEnd(ElementEnd::Empty) => {
                if let Some(circle) = circle.take() {
                    match circle {
                        (true, Some(x), Some(y), _) => {
                            if start.is_some() {
                                println!("WARGNING: svg map redefinition of start");
                            }
                            start = Some([x, y]);
                        }
                        (false, Some(x), Some(y), Some(r)) => bodies.push(Body {
                            pos: [x, y],
                            shape: Shape::Circle(r),
                        }),
                        _ => println!("WARGNING: svg map incomplete circle definition"),
                    }
                } else if let Some(rect) = rect.take() {
                    match rect {
                        (Some(x), Some(y), Some(w), Some(h)) => bodies.push(Body {
                            pos: [x+w/2., y-h/2.],
                            shape: Shape::Rectangle(w, h),
                        }),
                        _ => println!("WARGNING: svg map incomplete rect definition"),
                    }
                }
            },
            svg::Token::Attribute(svg::QName { local: svg::Name::Svg(AttributeId::Id), .. }, value) => {
                if let Some(ref mut circle) = circle {
                    circle.0 = value.to_str() == "start";
                }
            },
            svg::Token::Attribute(svg::QName { local: svg::Name::Svg(AttributeId::Cx), .. }, value) => {
                if let Some(ref mut circle) = circle {
                    circle.1 = Some(value.to_str().parse()?);
                }
            },
            svg::Token::Attribute(svg::QName { local: svg::Name::Svg(AttributeId::Cy), .. }, value) => {
                if let Some(ref mut circle) = circle {
                    circle.2 = Some(-value.to_str().parse()?);
                }
            },
            svg::Token::Attribute(svg::QName { local: svg::Name::Svg(AttributeId::R), .. }, value) => {
                if let Some(ref mut circle) = circle {
                    circle.3 = Some(value.to_str().parse()?);
                }
            },
            svg::Token::Attribute(svg::QName { local: svg::Name::Svg(AttributeId::X), .. }, value) => {
                if let Some(ref mut rect) = rect {
                    rect.0 = Some(value.to_str().parse()?);
                }
            },
            svg::Token::Attribute(svg::QName { local: svg::Name::Svg(AttributeId::Y), .. }, value) => {
                if let Some(ref mut rect) = rect {
                    rect.1 = Some(-value.to_str().parse()?);
                }
            },
            svg::Token::Attribute(svg::QName { local: svg::Name::Svg(AttributeId::Width), .. }, value) => {
                if let Some(ref mut rect) = rect {
                    rect.2 = Some(value.to_str().parse()?);
                }
            },
            svg::Token::Attribute(svg::QName { local: svg::Name::Svg(AttributeId::Height), .. }, value) => {
                if let Some(ref mut rect) = rect {
                    rect.3 = Some(value.to_str().parse()?);
                }
            },
            _ => (),
        }
    }

    Ok(Map {
        bodies: bodies,
        start: start.unwrap_or_else(|| {
            println!("WARGNING: svg map no start definition");
            [0., 0.]
        }),
    })
}

lazy_static! {
    pub static ref MAP: Map = load_map().ok_or_exit();
}

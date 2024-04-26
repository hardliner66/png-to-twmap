use std::{collections::HashMap, path::PathBuf};

use clap::{Parser, ValueEnum};
use image::{imageops::FilterType, DynamicImage, GenericImageView, Pixel, Pixels};
use kdtree::distance::squared_euclidean;
use kdtree::KdTree;
use ndarray::Array2;
use serde::{Deserialize, Serialize};
use twmap::{GameLayer, GameTile, TileFlags, TilemapLayer, TwMap};

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    print_default_mappings: bool,
    #[arg(short, long)]
    /// The mappings to use for the conversion
    mappings: Option<PathBuf>,
    #[arg(short, long, default_value = "1")]
    /// The size per tile
    tile_size: u32,
    /// The filter to use when resizing
    #[arg(short, long, default_value = "nearest")]
    resize_filter_type: ResizeFilterType,
    /// The path to the input PNG file
    input: Vec<PathBuf>,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum ResizeFilterType {
    /// Nearest Neighbor
    Nearest,
    /// Linear Filter
    Triangle,
    /// Cubic Filter
    CatmullRom,
    /// Gaussian Filter
    Gaussian,
    /// Lanczos with window 3
    Lanczos3,
}

impl ResizeFilterType {
    fn into_filter_type(self) -> FilterType {
        match self {
            ResizeFilterType::Nearest => FilterType::Nearest,
            ResizeFilterType::Triangle => FilterType::Triangle,
            ResizeFilterType::CatmullRom => FilterType::CatmullRom,
            ResizeFilterType::Gaussian => FilterType::Gaussian,
            ResizeFilterType::Lanczos3 => FilterType::Lanczos3,
        }
    }
}

pub fn export(
    path: &PathBuf,
    width: usize,
    height: usize,
    pixels: Pixels<DynamicImage>,
    mappings: &[([u8; 4], BlockType)],
    kdtree: &KdTree<f32, usize, [f32; 4]>,
) {
    let mut map = TwMap::parse(include_bytes!("../EMPTY.map")).expect("parsing failed");
    map.load().expect("loading failed");

    // get game layer
    let game_layer = map
        .find_physics_layer_mut::<GameLayer>()
        .unwrap()
        .tiles_mut()
        .unwrap_mut();

    *game_layer =
        Array2::<GameTile>::from_elem((height, width), GameTile::new(0, TileFlags::empty()));

    // modify game layer
    for (x, y, value) in pixels {
        let rgba = value.to_rgba();
        let channels = rgba.channels();
        let nearest_point = kdtree
            .nearest(
                &[
                    channels[0] as f32 / 255.0,
                    channels[1] as f32 / 255.0,
                    channels[2] as f32 / 255.0,
                    channels[3] as f32 / 255.0,
                ],
                1,
                &squared_euclidean,
            )
            .unwrap()[0]
            .1;
        let point = mappings[*nearest_point];
        game_layer[[y as usize, x as usize]] = GameTile::new(point.1.get_id(), TileFlags::empty());
    }

    // save map
    println!("exporting map to {:?}", &path);
    map.save_file(path).expect("saving failed");
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum BlockType {
    Empty,
    Hookable,
    Unhookable,
    Freeze,
    Spawn,
    Start,
    Finish,
    Custom(u8),
}

impl BlockType {
    fn get_id(self) -> u8 {
        match self {
            BlockType::Empty => 0,
            BlockType::Hookable => 1,
            BlockType::Unhookable => 2,
            BlockType::Freeze => 9,
            BlockType::Spawn => 192,
            BlockType::Start => 33,
            BlockType::Finish => 34,
            BlockType::Custom(id) => id,
        }
    }
}

type Mappings = HashMap<[u8; 4], BlockType>;

#[derive(Serialize, Deserialize)]
struct Mapping {
    mapping: Mappings,
}

const DEFAULT_MAPPING: &str = include_str!("../config.rsn");

fn parse_mappings(mappings: Option<PathBuf>) -> Mappings {
    mappings.map_or_else(
        || rsn::from_str(DEFAULT_MAPPING).expect("Failed to parse config"),
        |path| {
            rsn::from_str(&std::fs::read_to_string(path).expect("Config file doesn't exist"))
                .expect("Failed to parse config")
        },
    )
}

fn create_map(
    mappings: &[([u8; 4], BlockType)],
    kdtree: &KdTree<f32, usize, [f32; 4]>,
    tile_size: u32,
    resize_filter_type: ResizeFilterType,
    input_png: &PathBuf,
) {
    // Open the image file
    let mut img = image::open(input_png).expect("Failed to open image");

    // Get the dimensions of the image
    let (mut width, mut height) = img.dimensions();

    if tile_size > 1 {
        width /= tile_size;
        height /= tile_size;
        img = img.resize(width, height, resize_filter_type.into_filter_type());
    }

    // Determine output file name based on input or provided name
    let output_map = input_png.with_extension("map");

    export(
        &output_map,
        width as usize,
        height as usize,
        img.pixels(),
        mappings,
        kdtree,
    );
}

fn main() {
    let args = Args::parse();
    if args.print_default_mappings {
        println!("{DEFAULT_MAPPING}");
        return;
    }
    let mappings: Vec<([u8; 4], BlockType)> = parse_mappings(args.mappings).into_iter().collect();

    let mut kdtree = KdTree::new(4);
    for (i, ([r, g, b, a], _)) in mappings.iter().enumerate() {
        kdtree
            .add(
                [
                    *r as f32 / 255.0,
                    *g as f32 / 255.0,
                    *b as f32 / 255.0,
                    *a as f32 / 255.0,
                ],
                i,
            )
            .unwrap();
    }

    for input_png in args.input {
        create_map(
            &mappings,
            &kdtree,
            args.tile_size,
            args.resize_filter_type,
            &input_png,
        );
    }
}

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use clap::{Parser, ValueEnum};
use image::{DynamicImage, GenericImageView, Pixel, Pixels, imageops::FilterType};
use ndarray::Array2;
use serde::{Deserialize, Serialize};
use twmap::{GameLayer, GameTile, TileFlags, TilemapLayer, TwMap};

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
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
            ResizeFilterType::Nearest =>FilterType::Nearest,
            ResizeFilterType::Triangle =>FilterType::Triangle,
            ResizeFilterType::CatmullRom =>FilterType::CatmullRom,
            ResizeFilterType::Gaussian =>FilterType::Gaussian,
            ResizeFilterType::Lanczos3 =>FilterType::Lanczos3,
        }
    }
}

#[derive(Parser)]
struct GlobalOptions {
    #[arg(short, long)]
    /// The mappings to use for the conversion
    mappings: Option<PathBuf>,
    #[arg(short, long, default_value = "1")]
    /// The size per tile
    tile_size: u32,
    /// The filter to use when resizing
    #[arg(short, long, default_value = "nearest")]
    resize_filter_type: ResizeFilterType,
}

#[derive(Parser)]
enum Command {
    /// Prints the default mappings to stdout
    ExportMappings,
    /// Converts a PNG to a tw map
    Convert {
        #[clap(flatten)]
        global: GlobalOptions,
        /// The path to the input PNG file
        input_png: PathBuf,
        #[arg(short, long)]
        /// The path where the output map will be saved
        output_map: Option<PathBuf>,
    },
    /// Converts all PNGs in a directory to tw maps
    ConvertDirectory {
        #[clap(flatten)]
        global: GlobalOptions,
        /// Path to the input directory containing PNG files
        input_directory: PathBuf,
        /// Path to the output directory where converted maps will be saved
        #[arg(short, long)]
        output_directory: Option<PathBuf>,
    },
}

pub fn export<S: ::std::hash::BuildHasher>(
    path: &PathBuf,
    width: usize,
    height: usize,
    pixels: Pixels<DynamicImage>,
    mapping: &HashMap<[u8; 4], BlockType, S>,
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
        game_layer[[y as usize, x as usize]] = GameTile::new(
            mapping
                .get(&[channels[0], channels[1], channels[2], channels[3]])
                .unwrap_or(&BlockType::Empty)
                .get_id(),
            TileFlags::empty(),
        );
    }

    // save map
    println!("exporting map to {:?}", &path);
    map.save_file(path).expect("saving failed");
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum BlockType {
    Empty,
    Hookable,
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

fn create_map(mappings: &Mappings, tile_size: u32, resize_filter_type: ResizeFilterType, input_png: &PathBuf, output_map: Option<PathBuf>) {
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
    let output_map = output_map.unwrap_or_else(|| input_png.with_extension("map"));

    export(
        &output_map,
        width as usize,
        height as usize,
        img.pixels(),
        mappings,
    );
}

fn main() {
    let args = Args::parse();
    match args.command {
        Command::ExportMappings => {
            println!("{DEFAULT_MAPPING}");
        }
        Command::Convert {
            global,
            input_png,
            output_map,
        } => {
            let mappings: HashMap<[u8; 4], BlockType> = parse_mappings(global.mappings);
            create_map(&mappings, global.tile_size, global.resize_filter_type, &input_png, output_map);
        }

        Command::ConvertDirectory {
            global,
            input_directory,
            output_directory,
        } => {
            let mappings: HashMap<[u8; 4], BlockType> = parse_mappings(global.mappings);

            let output_directory = output_directory.unwrap_or_else(|| input_directory.clone());

            if std::fs::create_dir_all(&output_directory).is_err() {
                println!("Failed to create output directory.");
                return;
            }

            for entry in std::fs::read_dir(input_directory).expect("Failed to read input directory")
            {
                let entry = entry.expect("Failed to read entry");
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |e| e == "png") {
                    let file_name = path.file_name().unwrap();
                    let output_map =
                        output_directory.join(Path::new(file_name).with_extension("map"));

                    create_map(&mappings, global.tile_size, global.resize_filter_type, &path, Some(output_map));
                }
            }
        }
    }
}

use criterion::{black_box, criterion_group, criterion_main, Criterion, SamplingMode};
use itertools::Itertools;
use ply_rs::parser;
use ply_rs::ply;
use rand::Rng;
use std::{fs::create_dir_all, io::BufWriter, path::Path, time::SystemTime};
use uniform_grid::point_object::PointObject;
use uniform_grid::{spiral_cells, UniformGrid};

#[derive(Debug)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

impl Vertex {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Vertex { x, y, z }
    }
}

impl PointObject for Vertex {
    fn position(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}

impl ply::PropertyAccess for Vertex {
    fn new() -> Self {
        Vertex {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    fn set_property(&mut self, key: String, property: ply::Property) {
        match (key.as_ref(), property) {
            ("x", ply::Property::Float(v)) => self.x = v,
            ("y", ply::Property::Float(v)) => self.y = v,
            ("z", ply::Property::Float(v)) => self.z = v,
            (k, _) => panic!("Vertex: Unexpected key/value combination: key: {}", k),
        }
    }
}

fn ply_vertices(ply_path: &str) -> Vec<Vertex> {
    let file = std::fs::File::open(ply_path).unwrap();
    let mut reader = std::io::BufReader::new(file);
    let vertex_parser = parser::Parser::<Vertex>::new();

    let header = vertex_parser.read_header(&mut reader).unwrap();
    let mut vertices = Vec::new();

    for (_, element) in &header.elements {
        if element.name == "vertex" {
            vertices = vertex_parser
                .read_payload_for_element(&mut reader, element, &header)
                .unwrap();
        }
    }

    vertices
}

fn remove_multiple_random<T>(vec: &mut Vec<T>, count: usize) -> Vec<T> {
    let mut rng = rand::thread_rng();
    let mut removed = vec![];
    while removed.len() < count && !vec.is_empty() {
        let i = rng.gen_range(0..vec.len());
        removed.push(vec.remove(i));
    }
    removed
}

pub fn bench_dragon(c: &mut Criterion) {
    use std::time::Instant;

    let spiral = spiral_cells::spiral_cells(100);
    let (idx, cell) = spiral
        .iter()
        .enumerate()
        .max_by_key(|(i, sc)| sc.stop_cell_index1 - i)
        .unwrap();

    let diffs = spiral
        .iter()
        .enumerate()
        .map(|(i, sc)| sc.stop_cell_index1 - i)
        .take(100)
        .collect_vec();
    println!("Diffs: {:#?}", diffs);
    // println!("Stop-cell max diff: {}", (cell.stop_cell_index1 - idx));
    // println!(
    //     "Stop-cell mean diff: {}",
    //     (spiral_sum as f32 / spiral.len() as f32)
    // );

    let mut vertices = ply_vertices("./benches/data/dragon_vrip.ply");
    let queries = remove_multiple_random(&mut vertices, 10000);
    let offset_queries = queries
        .iter()
        .map(|v| Vertex::new(v.x * 0.7, v.y, v.z * 0.7))
        .collect_vec();
    let spiral = spiral_cells::read("./resources/spiral_100");

    let now = Instant::now();
    let uniform_grid = UniformGrid::new(vertices, 1.19, spiral);
    let elapsed = now.elapsed();
    println!("Pre-Processing time: {:.2?}", elapsed);

    let now = Instant::now();
    queries.iter().for_each(|q| {
        let _ = uniform_grid.nearest_neighbor(q.position());
    });
    let elapsed = now.elapsed();
    println!("Query time on surface: {:.2?}", elapsed);

    let now = Instant::now();
    offset_queries.iter().for_each(|q| {
        let _ = uniform_grid.nearest_neighbor(q.position());
    });
    let elapsed = now.elapsed();
    println!("Query time offset from surface: {:.2?}", elapsed);
}

criterion_group!(benches, bench_dragon);
criterion_main!(benches);

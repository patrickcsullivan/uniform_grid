use criterion::{black_box, criterion_group, criterion_main, Criterion, SamplingMode};
use ply_rs::parser;
use ply_rs::ply;
use std::{fs::create_dir_all, io::BufWriter, path::Path, time::SystemTime};
use uniform_grid::{spiral_cells, UniformGrid};

struct Vertex {
    x: f32,
    y: f32,
    z: f32,
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

pub fn bench_dragon(c: &mut Criterion) {
    use std::time::Instant;
    let now = Instant::now();
    let vertices = ply_vertices("./benches/data/dragon_vrip.ply");
    let elapsed = now.elapsed();
    println!("Vertices: {}", vertices.len());
    println!("Elapsed: {:.2?}", elapsed);
}

criterion_group!(benches, bench_dragon);
criterion_main!(benches);

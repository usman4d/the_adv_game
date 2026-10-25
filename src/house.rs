use bevy::{asset::RenderAssetUsages, mesh::{Indices, PrimitiveTopology}, prelude::*};
use bevy_rapier3d::prelude::*;
use geometry::derivator::Derivator;
use geometry::rule::{Rule, RuleEvaluator};
use geometry::scope::{Direction, Face, Scope};
// The derive macros expand to code that names these items directly.
use geometry::symbol::{Symbol, SymbolData};
use rand::{distributions::WeightedIndex, prelude::*};

use crate::ground::SnapToGround;

/// Spawns a procedurally generated house near the player's spawn point.
pub struct HousePlugin;

/// Where the house goes, a short walk in front of the player's spawn.
const HOUSE_POSITION: Vec3 = Vec3::new(-26.0, 0.0, -40.0);

impl Plugin for HousePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_house);
    }
}

fn spawn_house(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let axiom = AxiomSymbol(SymbolData {
        scope: Scope::default(),
        is_terminal: false,
    });
    let geometry_data = Derivator::new(axiom).derive();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, geometry_data.verticies);
    mesh.insert_indices(Indices::U32(geometry_data.indicies));
    let collider = Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh(TriMeshFlags::default()))
        .expect("Couldn't convert house mesh into a collider");
    mesh.duplicate_vertices();
    mesh.compute_flat_normals();

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(Color::srgb(0.82, 0.74, 0.62))),
        Transform::from_translation(HOUSE_POSITION),
        RigidBody::Fixed,
        collider,
        // Sink the foundation slightly so uneven terrain doesn't leave gaps.
        SnapToGround { height: -0.5 },
    ));
}

#[derive(Clone, macros::Symbol, macros::RuleEvaluator)]
#[rules(Axiom)]
struct AxiomSymbol(SymbolData);

#[derive(Clone, macros::Symbol, macros::RuleEvaluator)]
#[rules(HouseSingleRoom, HouseDualRoom, HouseTriRoom)]
struct HouseSymbol(SymbolData);

#[derive(Clone, macros::Symbol, macros::RuleEvaluator)]
#[rules(RoofSimple)]
struct RoofSymbol(SymbolData);

#[derive(Clone, macros::Symbol, macros::RuleEvaluator)]
#[rules(Facade)]
struct FacadeSymbol(SymbolData);

struct Axiom;
struct HouseSingleRoom;
struct HouseDualRoom;
struct HouseTriRoom;
struct RoofSimple;
struct Facade;

fn split_or_whole(result: Result<Vec<Scope>, Scope>) -> Vec<Scope> {
    result.unwrap_or_else(|whole| {
        warn!("split operation failed");
        vec![whole]
    })
}

fn facade(scope: Scope) -> Box<dyn RuleEvaluator> {
    Box::new(FacadeSymbol(SymbolData {
        scope,
        is_terminal: true,
    }))
}

fn roof(scope: Scope) -> Box<dyn RuleEvaluator> {
    Box::new(RoofSymbol(SymbolData {
        scope,
        is_terminal: false,
    }))
}

/// Splits a house body into rooms along X; `raised` picks the room whose
/// upper floor is extruded by `raise` before its roof is added.
fn rooms(scope: &Scope, spec: &str, ratios: &[f32], raised: usize, raise: f32) -> Vec<Box<dyn RuleEvaluator>> {
    let mut body = scope.clone();
    body.extrude(6.0);
    let mut symbols = Vec::new();
    for (index, room) in split_or_whole(body.split(Direction::X, spec, ratios)).into_iter().enumerate() {
        let mut upper = room.get_face(Face::Top);
        upper.extrude(if index == raised { raise } else { 0.0 });
        symbols.push(roof(upper.get_face(Face::Top)));
        symbols.push(facade(room));
        symbols.push(facade(upper));
    }
    symbols
}

impl Rule<Axiom> for AxiomSymbol {
    fn evaluate(&mut self) -> Option<Vec<Box<dyn RuleEvaluator>>> {
        let mut lot = Scope::default();
        lot.set_size(12.0, 0.0, 8.0);
        Some(vec![Box::new(HouseSymbol(SymbolData {
            scope: lot,
            is_terminal: false,
        }))])
    }
    fn is_terminal(&self) -> bool {
        self.get_data().is_terminal
    }
    fn scope(&self) -> Scope {
        self.get_data().scope.clone()
    }
    fn probability() -> f32 {
        1.0
    }
}

impl Rule<HouseSingleRoom> for HouseSymbol {
    fn evaluate(&mut self) -> Option<Vec<Box<dyn RuleEvaluator>>> {
        self.get_data_mut().scope.extrude(6.0);
        let scope = self.get_data().scope.clone();
        Some(vec![roof(scope.get_face(Face::Top)), facade(scope)])
    }
    fn is_terminal(&self) -> bool {
        self.get_data().is_terminal
    }
    fn scope(&self) -> Scope {
        self.get_data().scope.clone()
    }
    fn probability() -> f32 {
        1.0
    }
}

impl Rule<HouseDualRoom> for HouseSymbol {
    fn evaluate(&mut self) -> Option<Vec<Box<dyn RuleEvaluator>>> {
        Some(rooms(&self.get_data().scope, "rr", &[2.0, 1.0], 0, 1.0))
    }
    fn is_terminal(&self) -> bool {
        self.get_data().is_terminal
    }
    fn scope(&self) -> Scope {
        self.get_data().scope.clone()
    }
    fn probability() -> f32 {
        1.0
    }
}

impl Rule<HouseTriRoom> for HouseSymbol {
    fn evaluate(&mut self) -> Option<Vec<Box<dyn RuleEvaluator>>> {
        Some(rooms(&self.get_data().scope, "rrr", &[1.1, 1.0, 1.1], 1, 3.0))
    }
    fn is_terminal(&self) -> bool {
        self.get_data().is_terminal
    }
    fn scope(&self) -> Scope {
        self.get_data().scope.clone()
    }
    fn probability() -> f32 {
        1.0
    }
}

impl Rule<RoofSimple> for RoofSymbol {
    fn evaluate(&mut self) -> Option<Vec<Box<dyn RuleEvaluator>>> {
        self.get_data_mut().scope.extrude(1.0);
        let splits = split_or_whole(self.get_data().scope.split(Direction::X, "ara", &[0.5, 1.0, 0.5]));

        let mut facades = Vec::with_capacity(5);
        for (s, split) in splits.iter().enumerate() {
            if s == 1 {
                let mut inner = split_or_whole(split.split(Direction::Z, "ara", &[0.5, 1.0, 0.5]));
                for (y, part) in inner.iter_mut().enumerate() {
                    if y == 1 {
                        part.extrude(-0.5);
                    }
                    facades.push(facade(part.clone()));
                }
            } else {
                facades.push(facade(split.clone()));
            }
        }
        Some(facades)
    }
    fn is_terminal(&self) -> bool {
        self.get_data().is_terminal
    }
    fn scope(&self) -> Scope {
        self.get_data().scope.clone()
    }
    fn probability() -> f32 {
        1.0
    }
}

impl Rule<Facade> for FacadeSymbol {
    fn evaluate(&mut self) -> Option<Vec<Box<dyn RuleEvaluator>>> {
        None
    }
    fn is_terminal(&self) -> bool {
        self.get_data().is_terminal
    }
    fn scope(&self) -> Scope {
        self.get_data().scope.clone()
    }
    fn probability() -> f32 {
        1.0
    }
}

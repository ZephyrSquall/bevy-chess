use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            // Prevent anti-aliasing
            .set(ImagePlugin::default_nearest()),
    )
    .add_systems(Startup, setup)
    .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Rectangle::new(50.0, 50.0))),
        material: materials.add(Color::linear_rgb(1.0, 1.0, 1.0)),
        transform: Transform::from_xyz(100.0, 0.0, 0.0),
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/king_white.png"),
        transform: Transform {
            translation: Vec3 {
                x: 100.0,
                y: 100.0,
                z: 0.0,
            },
            scale: Vec3::splat(10.0),
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/king_black.png"),
        transform: Transform {
            translation: Vec3 {
                x: 100.0,
                y: 0.0,
                z: 0.0,
            },
            scale: Vec3::splat(10.0),
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/pawn_white.png"),
        transform: Transform {
            translation: Vec3 {
                x: 200.0,
                y: 100.0,
                z: 0.0,
            },
            scale: Vec3::splat(10.0),
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/pawn_black.png"),
        transform: Transform {
            translation: Vec3 {
                x: 250.0,
                y: 100.0,
                z: 0.0,
            },
            scale: Vec3::splat(10.0),
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/knight_white.png"),
        transform: Transform {
            translation: Vec3 {
                x: 250.0,
                y: 260.0,
                z: 0.0,
            },
            scale: Vec3::splat(10.0),
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/knight_black.png"),
        transform: Transform {
            translation: Vec3 {
                x: 250.0,
                y: 200.0,
                z: 0.0,
            },
            scale: Vec3::splat(10.0),
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/rook_white.png"),
        transform: Transform {
            translation: Vec3 {
                x: -250.0,
                y: -200.0,
                z: 0.0,
            },
            scale: Vec3::splat(10.0),
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/rook_black.png"),
        transform: Transform {
            translation: Vec3 {
                x: -200.0,
                y: -200.0,
                z: 0.0,
            },
            scale: Vec3::splat(10.0),
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/bishop_white.png"),
        transform: Transform {
            translation: Vec3 {
                x: 200.0,
                y: -200.0,
                z: 0.0,
            },
            scale: Vec3::splat(10.0),
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/bishop_black.png"),
        transform: Transform {
            translation: Vec3 {
                x: 200.0,
                y: -250.0,
                z: 0.0,
            },
            scale: Vec3::splat(10.0),
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/queen_white.png"),
        transform: Transform {
            translation: Vec3 {
                x: -200.0,
                y: 200.0,
                z: 0.0,
            },
            scale: Vec3::splat(10.0),
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("pieces/queen_black.png"),
        transform: Transform {
            translation: Vec3 {
                x: -200.0,
                y: 250.0,
                z: 0.0,
            },
            scale: Vec3::splat(10.0),
            ..default()
        },
        ..default()
    });
}

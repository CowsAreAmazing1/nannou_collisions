
use std::time::Duration;
use nannou::prelude::*;


use solver::solver::*;
pub mod solver;



const DT: f32 = 0.1;


fn main() {
    nannou::app(model).update(update).loop_mode(LoopMode::refresh_sync()).run()
}



struct Model {
    solver: PhysicsSolver,
    window: WindowId,
    prev_pos: Vec2,
    show_quadtree: bool,
    spawn_objects: bool,
}

fn model(app: &App) -> Model {
    let window = app.new_window()
        .title(app.exe_name().unwrap())
        .size(1600, 1000)
        .view(view)
        .key_pressed(key_pressed)
        .build()
        .unwrap();

    let ps = PhysicsSolver::new(app, 0.1, 8);

    Model {
        solver: ps,
        window,
        prev_pos: Vec2::ZERO,
        show_quadtree: false,
        spawn_objects: true,
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    let window_vel = app.window(model.window).unwrap().outer_position_pixels().unwrap_or_default();
    let window_vel = vec2(window_vel.0 as f32, window_vel.1 as f32);

    model.solver.update(&DT, -0.003 * (model.prev_pos - window_vel));
    model.prev_pos = window_vel;


    if update.since_last < Duration::from_millis(17) && model.spawn_objects {
        let s = 8.0;
        let pos = vec2(model.solver.world.w() / s, (s-1.0) * model.solver.world.h() / s); // model.solver.world.top_left();
        let radius = 10.0;
        let hue = update.since_start.as_millis() as f32;
        let mut ball1 = Ball::new(pos, radius, hue / 50.0);

        let osc = (hue / 500.0).cos();
        let angle = osc * 0.1;
        let speed = radius * 0.05 / DT;
        ball1.add_velocity(speed * vec2(angle.cos(), angle.sin()));

        for i in 0..6 {
            ball1.set_position_same_speed(pos + 2.0 * radius * i as f32 * vec2(angle.sin(), -angle.cos()));
            model.solver.add_object(ball1.clone());
        }
    }
}

fn draw_quadtree(draw: &Draw, node: &QuadTreeNode) {
    // Draw this node's bounds
    let rect = node.bounds;
    draw.rect()
        .xy(rect.xy())
        .w_h(rect.w(), rect.h())
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(0.5);
    if let Some(children) = &node.children {
        for child in children.iter() {
            draw_quadtree(draw, child);
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    // println!("{}", 4.0f32.powi(QT_MAX_DEPTH as i32) * model.solver.quadtree.average_objects_per_node());

    let (w, h) = app.window_rect().w_h();
    let draw = app.draw().x_y(-w/2.0, -h/2.0);
    draw.background().color(BLACK);

    if model.show_quadtree {
        draw_quadtree(&draw, &model.solver.quadtree.root);
    }

    for obj in &model.solver.objects {
        let cdraw = draw.xy(obj.position);
        cdraw.ellipse()
            .color(obj.color())
            // .no_fill()
            // .stroke(obj.color)
            // .stroke_weight(1.0)
            // .w_h(1.0, 1.0)
            .radius(obj.radius)
            .resolution(8.0)
            ;
    }

    draw.to_frame(app, &frame).unwrap();
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::R => {
            model.solver.objects.clear();
        }
        Key::Q => {
            model.show_quadtree = !model.show_quadtree;
        }
        Key::Space => {
            model.spawn_objects = !model.spawn_objects;
        }
        Key::S => {
            model.solver.freeze();
        }
        _other_key => {}
    }
}
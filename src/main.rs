
use nannou::prelude::*;

use solver::solver::*;
pub mod solver;

const DT: f32 = 0.1;
const RADIUS: f32 = 5.0;

fn main() {
    nannou::app(model).update(update).loop_mode(LoopMode::refresh_sync()).run()
}


struct Model {
    solver: PhysicsSolver,
    window: WindowId,
    prev_pos: Vec2,
    show_quadtree: bool,
    spawn_objects: bool,
    use_wall_clock_dt: bool,
    mouse_spawner: Option<Vec2>,
}

fn model(app: &App) -> Model {
    let window = app.new_window()
        .title(app.exe_name().unwrap())
        .size(1600, 1000)
        .view(view)
        .key_pressed(key_pressed)
        .resized(window_resized)
        .build()
        .unwrap();

    let ps = PhysicsSolver::new(app, 0.0, 4);

    Model {
        solver: ps,
        window,
        prev_pos: Vec2::ZERO,
        show_quadtree: true,
        spawn_objects: false,
        use_wall_clock_dt: false,
        mouse_spawner: None,
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    let window_vel = app.window(model.window).unwrap().outer_position_pixels().unwrap_or_default();
    let window_vel = vec2(window_vel.0 as f32, window_vel.1 as f32);

    model.solver.update(&DT, -0.003 * (model.prev_pos - window_vel));
    model.prev_pos = window_vel;

    if model.spawn_objects { // & update.since_last < Duration::from_millis(17) {
        let s = 6.0;
        let pos = vec2(model.solver.world.w() / s, (s-1.0) * model.solver.world.h() / s); // model.solver.world.top_left();

        let osc = update.since_start.as_millis() as f32;
        
        let angle = 0.1 * (osc * 0.0005).sin();
        let dir = vec2(angle.cos(), angle.sin()).normalize();
        let margin = 0.5 * RADIUS;
        let a_parallel = model.solver.gravity.dot(dir);
        let d_clear = 2.0 * RADIUS + margin;
        let v_min = ((d_clear - 0.5 * a_parallel * DT * DT) / DT).max(0.0);
        let velocity = v_min * dir;

        let sub_dt = DT / model.solver.sub_steps as f32;
        let hue = osc / 50.0;
        
        for i in 0..5 {
            let ball_pos = pos + 2.0 * RADIUS * (i-15) as f32 * vec2(-angle.sin(), angle.cos());
            model.solver.add_object(Ball::with_velocity(
                ball_pos,
                velocity,
                RADIUS,
                hue,
                sub_dt
            ));
        }
    }

    if app.mouse.buttons.left().is_down() {
        let mid = vec2(app.window_rect().w()/2.0, app.window_rect().h()/2.0);

        if let Some(emit_pos) = model.mouse_spawner {
            let o2o1 = app.mouse.position() - emit_pos;
            let dist = o2o1.length();
            let dir = o2o1 / dist;
            let angle = o2o1.angle();

            let margin = 0.5 * RADIUS;
            let a_parallel = model.solver.gravity.dot(dir);
            let d_clear = 2.0 * RADIUS + margin;
            let v_min = ((d_clear - 0.5 * a_parallel * DT * DT) / DT).max(0.0);
            let velocity = dist * v_min * dir * 0.01;

            let sub_dt = DT / model.solver.sub_steps as f32;
            
            for i in 0..1 {
                let ball_pos = mid + emit_pos + 2.0 * RADIUS * (i) as f32 * vec2(-angle.sin(), angle.cos());
                model.solver.add_object(Ball::with_velocity(
                    ball_pos,
                    velocity,
                    RADIUS,
                    0.0,
                    sub_dt
                ));
            }

            // model.mouse_spawner = Some(app.mouse.position());
        } else {
            let mouse_pos = app.mouse.position();
            model.mouse_spawner = Some(mouse_pos);
        }
    } else {
        model.mouse_spawner = None;
    }
}

fn window_resized(_app: &App, model: &mut Model, new_size: Vec2) {
    model.solver.world = Rect::from_corners(pt2(0.0, 0.0), pt2(new_size.x, new_size.y));
    model.solver.refresh_quadtree();
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
            .no_fill()
            .stroke(obj.color())
            .stroke_weight(1.0)
            .w_h(1.0, 1.0)
            .radius(obj.radius)
            .resolution(4.0)
            ;
    }

    draw.text(model.solver.objects.len().to_string().as_str())
        .xy(vec2(500.0,500.0))
        .color(WHITE)
        .font_size(24);

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
        Key::T => {
            model.use_wall_clock_dt = !model.use_wall_clock_dt;
        }
        _other_key => {}
    }
}
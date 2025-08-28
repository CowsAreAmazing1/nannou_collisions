
use std::time::Duration;
use nannou::prelude::*;


use solver::solver::*;
pub mod solver;



const DT: f32 = 0.1;


fn main() {
    nannou::app(model).update(update).loop_mode(LoopMode::refresh_sync()).run()
}


#[derive(Clone)]
struct CollisionCell {
    width: u32,
    height: u32,
    objects: Vec<usize>,
}

impl CollisionCell {
    fn default() -> Self {
        CollisionCell {
            width: 0,
            height: 0,
            objects: Vec::new(),
        }
    }

    fn add_atom(&mut self, id: usize) {
        self.objects.push(id);
    }

    fn clear(&mut self) {
        self.objects = Vec::new();
    }

    // fn remove(&mut self, id: usize) {
    //     self.objects.remove(id);
    // } 
}

pub struct CollisionGrid {
    nx: u32,
    ny: u32,
    data: Vec<CollisionCell>,
}

impl CollisionGrid {
    fn new(width: f32, height: f32, nx: usize, ny: usize) -> Self {
        let mut data= vec![CollisionCell::default(); (nx * ny) as usize];

        for y in 0..nx {
            for x in 0..ny {
                data[y * ny as usize + x].width = (width / nx as f32) as u32;
                data[y * ny as usize + x].height = (height / ny as f32) as u32;
            }
        }

        CollisionGrid {
            nx: nx as u32,
            ny: ny as u32,
            data,
        }

    }

    fn get_w_vec(&self, v: Vec2) -> &CollisionCell {
        self.get(v.x as usize, v.y as usize)
    }

    fn get(&self, x: usize, y: usize) -> &CollisionCell {
        &self.data[(y as u32 * self.nx + x as u32) as usize]
    }

    // fn ind_to_vec(&self, ind: usize) -> Vec2 {
    //     vec2(ind as f32 % self.width as f32, (ind as f32 / self.width as f32).floor())
    // }

    // fn set(&mut self, x: usize, y: usize, id: usize) {
    //     self.data[y * self.height as usize + x].add_atom(id);
    // }

    fn clear(&mut self) {
        for i in &mut self.data {
            i.clear();
        }
    }

}




struct Model {
    solver: PhysicsSolver,
}

impl Model {
    fn coord_change(&self, pos: Vec2) -> Vec2 {
        return pos - self.solver.world.top_right()
    }
}

fn model(app: &App) -> Model {
    let _window = app.new_window()
        .title(app.exe_name().unwrap())
        .size(1500, 300)
        .view(view)
        .key_pressed(key_pressed)
        .build()
        .unwrap();

    let ps = PhysicsSolver::new(
        app, 3.0, 10.0, 8
    );

    Model {
        solver: ps,
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    model.solver.update(&DT);

    if update.since_last < Duration::from_millis(100) {
        let pos = vec2(model.solver.world.w() / 2.0, model.solver.world.h() / 2.0); // model.solver.world.top_left();
        let radius = 3.0; //random_range(5.0, 10.0); // 
        let hue = update.since_start.as_millis() as f32 / 30.0;
        let mut ball1 = Ball::new(pos, radius, hue);
        ball1.add_velocity(radius / 200.0 * vec2((hue / 50.0).cos(), (hue / 50.0).sin()));
        //ball1.add_velocity( vec2(radius / 2.0, 0.0));

        // ball1.set_position_same_speed(pos + vec2(20.0, -50.0 - 2.5 * radius));
        ball1.set_position_same_speed(pos);
        model.solver.add_object(ball1.clone());

        ball1.set_position_same_speed(pos + vec2(20.0, -55.0 - 5.0 * radius));
        model.solver.add_object(ball1.clone());

        ball1.set_position_same_speed(pos + vec2(20.0, -60.0 - 7.5 * radius));
        model.solver.add_object(ball1.clone());

        ball1.set_position_same_speed(pos + vec2(20.0, -65.0 - 10.0 * radius));
        model.solver.add_object(ball1.clone());

        ball1.set_position_same_speed(pos + vec2(20.0, -70.0 - 12.5 * radius));
        model.solver.add_object(ball1.clone());
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    for obj in &model.solver.objects {
        let cdraw = draw.xy(model.coord_change(obj.position));
        cdraw.ellipse()
            //.color(Hsv::from_components((30.0, 1.0, obj.get_speed())))
            .color(obj.color)
            //.no_fill()
            //.stroke(WHITE)
            //.stroke_weight(LINE_WIDTH)
            .w_h(1.0, 1.0)
            .radius(obj.radius)
            ;
    }

    // for i in 0..model.solver.grid.nx {
    //     let start_pt = vec2(model.solver.grid.data[0].width as f32 * i as f32, 0.0);
    //     let end_pt = vec2(model.solver.grid.data[0].width as f32 * i as f32, model.solver.grid.height as f32);

    //     draw.line()
    //         .start(model.coord_change(start_pt))
    //         .end(model.coord_change(end_pt))
    //         .weight(1.0)
    //         .color(WHITE);

    //     let start_pt = vec2(0.0, model.solver.grid.data[0].height as f32 * i as f32);
    //     let end_pt = vec2(model.solver.grid.width as f32, model.solver.grid.data[0].height as f32 * i as f32);

    //     draw.line()
    //         .start(model.coord_change(start_pt))
    //         .end(model.coord_change(end_pt))
    //         .weight(1.0)
    //         .color(WHITE);
    // }


    draw.to_frame(app, &frame).unwrap();
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::R => {
            model.solver.objects.clear();
        }
        _other_key => {}
    }
}




pub mod solver {

use nannou::prelude::*;
use nannou::color::Hsv;

use crate::CollisionGrid;


#[derive(Clone, Copy)]
pub struct Ball {
    pub position: Vec2,
    last_position: Vec2,
    pub acceleration: Vec2,
    pub radius: f32,
    pub color: Hsv,
}

impl Ball {
    pub fn new(pos: Vec2, radius: f32, hue: f32) -> Self {
        let acc = vec2(0.0, 0.0);
        let color = Hsv::from_components((hue, 1.0, 1.0));
        Ball {
            position: pos,
            last_position: pos,
            acceleration: acc,
            radius,
            color,
        }
    }

    // fn set_position(&mut self, pos: Vec2) {
    //     self.position      = pos;
    //     self.last_position = pos;
    // }

    pub fn update(&mut self, dt: &f32) {
        let last_update_move = self.position - self.last_position;
        let velocity_damping: f32 = 0.0;

        let new_position = self.position + last_update_move + (self.acceleration - last_update_move * velocity_damping) * (dt * dt);
        self.last_position = self.position;
        self.position      = new_position;
        self.acceleration = vec2(0.0, 0.0);

    }

    // fn stop(&mut self) {
    //     self.last_position = self.position;
    // }

    // fn slow_down(&mut self, ratio: &f32) {
    //     self.last_position = self.last_position + (self.position - self.last_position) * (ratio * 1.0);
    // }

    // fn get_speed(&self) -> f32 {
    //     return (self.position - self.last_position).length();
    // }

    // fn get_velocity(&mut self) -> Vec2 {
    //     return self.position - self.last_position;
    // }

    pub fn add_velocity(&mut self, v: Vec2) {
        self.last_position -= v;
    }

    pub fn set_position_same_speed(&mut self, new_position: Vec2) {
        let to_last = self.last_position - self.position;
        self.position = new_position;
        self.last_position = self.position + to_last;
    }

    // fn move_ball(&mut self, v: Vec2) {
    //     self.position += v;
    // }
}

pub struct PhysicsSolver {
    pub objects: Vec<Ball>,
    pub grid: CollisionGrid,
    pub world: Rect,
    pub gravity: Vec2,

    pub sub_steps: u32,
}

impl PhysicsSolver {
    pub fn new(app: &App, max_radius: f32, gravity_mag: f32, sub_steps: u32) -> Self {
        let objects: Vec<Ball> = Vec::new();
        let world = app.window_rect();

        let grid = CollisionGrid::new(
            world.w(), 
            world.h(), 
            (world.w()/(2.0*max_radius)).floor() as usize, 
            (world.h()/(2.0*max_radius)).floor() as usize
        );

        let gravity = vec2(0.0, gravity_mag);

        PhysicsSolver {
            objects,
            grid,
            world,
            gravity,
            sub_steps
        }


    }

    fn gridify(&mut self) {
        self.grid.clear();

        for i in 0..self.objects.len() {
            let pos = self.objects[i].position;

            let x = (pos.x / self.grid.data[0].width as f32).floor() as usize;
            let y = (pos.y / self.grid.data[0].height as f32).floor() as usize;

            //println!("Pos = {}, x = {}, y = {}, index = {}", pos, x, y, (y as u32 * self.grid.nx + x as u32) as usize);

            self.grid.data[(y as u32 * self.grid.nx + x as u32) as usize].add_atom(i);
        }
    }
    
    pub fn update(&mut self, dt: &f32) { // main update loop for the full simulation
        let sub_dt = dt / self.sub_steps as f32;

        for _ in 0..self.sub_steps {
            self.solve_collsions();
            self.update_other(&sub_dt);
        }
    }

    fn solve_collsions(&mut self) { // main collision solving loop for the full simulation
        self.gridify();

        for y in 1..self.grid.ny - 1 {
            for x in 1..self.grid.nx - 1 {
                self.process_cell(vec2(x as f32,y as f32));
            }
        }
    }

    fn process_cell(&mut self, cell_id: Vec2) { // processes a single cell for collisions with adjacent cells
        let mut current_cell_objs = self.grid.get_w_vec(cell_id).objects.to_owned();
        
        let offsets = vec![
            cell_id + vec2(-1.0, -1.0),
            cell_id + vec2( 0.0, -1.0),
            cell_id + vec2( 1.0, -1.0),
            cell_id + vec2(-1.0,  0.0),
            //cell_id,
            cell_id + vec2( 1.0,  0.0),
            cell_id + vec2(-1.0,  1.0),
            cell_id + vec2( 0.0,  1.0),
            cell_id + vec2( 1.0,  1.0),
        ];

        for i in &offsets {
            current_cell_objs.extend(self.grid.get_w_vec(*i).objects.to_owned());
        }

        for obj_id in 0..current_cell_objs.len() {
            for other_obj_id in 0..current_cell_objs.len() {
                if obj_id != other_obj_id {
                    self.solve_contact(current_cell_objs[obj_id], current_cell_objs[other_obj_id]);
                }
            }
        }
    }

    fn solve_contact(&mut self, b1_ind: usize, b2_ind: usize) { // Single ball on ball collision
        if b1_ind == b2_ind {
            return;
        }
        
        let response_coef = 1.0;
        let eps = 0.0001;
        
        let o2_o1 = self.objects[b1_ind].position - self.objects[b2_ind].position;
        let dist = self.objects[b1_ind].position.distance(self.objects[b2_ind].position);
        
        if dist < self.objects[b1_ind].radius + self.objects[b2_ind].radius && dist > eps {
            //println!("{} and {} out of {}", b1_ind, b2_ind, self.objects.len());
            let delta = response_coef * 0.5 * (self.objects[b1_ind].radius + self.objects[b2_ind].radius - dist);
            let col_vec = (o2_o1 / dist) * delta;
            self.objects[b1_ind].position += col_vec;
            self.objects[b2_ind].position -= col_vec;
        }
    }



    fn update_other(&mut self, dt: &f32) { // other updates for the full simulation
        for obj in &mut self.objects { 
            obj.acceleration -= self.gravity;
            obj.update(dt);

            if obj.position.y > self.world.h() - obj.radius {
                // obj.position.y = self.world.h() - obj.radius;
                obj.last_position.y = self.world.h() - obj.radius;
                obj.position.y = 2.0 * self.world.h() - 2.0 * obj.radius - obj.position.y;
            } else if obj.position.y < obj.radius {
                // obj.position.y = obj.radius;
                obj.last_position.y = obj.radius;
                obj.position.y = 2.0 * obj.radius - obj.position.y;
            }

            if obj.position.x > self.world.w() - obj.radius {
                // obj.position.x = self.world.w() - obj.radius;
                obj.last_position.x = self.world.w() - obj.radius;
                obj.position.x = 2.0 * self.world.w() - 2.0 * obj.radius - obj.position.x;
            } else if obj.position.x < obj.radius {
                // obj.position.x = obj.radius;
                obj.last_position.x = obj.radius;
                obj.position.x = 2.0 * obj.radius - obj.position.x;
            }
        }
    }

    

    pub fn add_object(&mut self, obj: Ball) {
        self.objects.push(obj);
    }
}

}

pub mod solver {


use nannou::prelude::*;
use nannou::color::Hsv;


pub struct Ball {
    pub position: Vec2,
    last_position: Vec2,
    pub acceleration: Vec2,
    pub radius: f32,
    hue: f32,
}

impl Ball {
    pub fn new(pos: Vec2, speed: Vec2, radius: f32, hue: f32) -> Self {
        Ball {
            position: pos,
            last_position: pos - speed,
            acceleration: Vec2::ZERO,
            radius,
            hue,
        }
    }

    // Construct a ball from a velocity vector and the integration timestep.
    // Ensures initial motion is consistent regardless of sub-stepping.
    pub fn with_velocity(pos: Vec2, velocity: Vec2, radius: f32, hue: f32, dt: f32) -> Self {
        Ball {
            position: pos,
            last_position: pos - velocity * dt,
            acceleration: Vec2::ZERO,
            radius,
            hue,
        }
    }

    pub fn color(&self) -> Hsv {
        Hsv::from_components((self.hue, 1.0, 1.0))
    }

    // fn set_position(&mut self, pos: Vec2) {
    //     self.position      = pos;
    //     self.last_position = pos;
    // }

    pub fn update(&mut self, dt: &f32) {
        let pos_diff = self.position - self.last_position;
        let velocity_damping: f32 = 0.0;

        let new_position = self.position + pos_diff + (self.acceleration - pos_diff * velocity_damping) * (dt * dt);
        self.last_position = self.position;
        self.position      = new_position;
        self.acceleration  = Vec2::ZERO;

        // self.hue = (self.hue + 0.05) % 360.0;
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

#[inline]
fn circle_intersects_rect(center: Vec2, radius: f32, rect: &Rect) -> bool {
    let closest_x = center.x.clamp(rect.left(), rect.right());
    let closest_y = center.y.clamp(rect.bottom(), rect.top());
    let dx = center.x - closest_x;
    let dy = center.y - closest_y;
    (dx * dx + dy * dy) <= radius * radius
}

pub struct QuadTreeNode {
    pub bounds: Rect,
    objects: Vec<(usize, Vec2, f32)>, // (index, position, radius)
    pub children: Option<[Box<QuadTreeNode>; 2]>,
    depth: usize,
}

impl QuadTreeNode {
    fn new(bounds: Rect, depth: usize) -> Self {
        Self {
            bounds,
            objects: Vec::with_capacity(QT_MAX_OBJECTS),
            children: None,
            depth,
        }
    }

    fn subdivide(&mut self) {
        let min = self.bounds.bottom_left();
        let max = self.bounds.top_right();
        let mid = (min + max) / 2.0;

        // // Order: [nw, ne, sw, se]
        // let nw = Rect::from_corners(pt2(min.x, mid.y), pt2(mid.x, max.y));
        // let ne = Rect::from_corners(pt2(mid.x, mid.y), pt2(max.x, max.y));
        // let sw = Rect::from_corners(min, mid);
        // let se = Rect::from_corners(pt2(mid.x, min.y), pt2(max.x, mid.y));

        // self.children = Some([
        //     Box::new(QuadTreeNode::new(nw, self.depth + 1)),
        //     Box::new(QuadTreeNode::new(ne, self.depth + 1)),
        //     Box::new(QuadTreeNode::new(sw, self.depth + 1)),
        //     Box::new(QuadTreeNode::new(se, self.depth + 1)),
        // ]);
        
        // Order: [left, right]
        // if self.depth.is_power_of_two() {
        if max.x - min.x > max.y - min.y {
            let left  = Rect::from_corners(min, pt2(mid.x, max.y));
            let right = Rect::from_corners(pt2(mid.x, min.y), max);
    
            self.children = Some([
                Box::new(QuadTreeNode::new(left, self.depth + 1)),
                Box::new(QuadTreeNode::new(right, self.depth + 1)),
            ]);
        } else {
            let bot = Rect::from_corners(min, pt2(max.x, mid.y));
            let top = Rect::from_corners(pt2(min.x, mid.y), max);
    
            self.children = Some([
                Box::new(QuadTreeNode::new(top, self.depth + 1)),
                Box::new(QuadTreeNode::new(bot, self.depth + 1)),
            ]);
        }
    }

    #[inline]
    fn insert(&mut self, index: usize, pos: Vec2, radius: f32) {
        if !circle_intersects_rect(pos, radius, &self.bounds) {
            return;
        }

        if let Some(children) = &mut self.children {
            for child in children.iter_mut() {
                child.insert(index, pos, radius);
            }
            return;
        }

        self.objects.push((index, pos, radius));

        if self.objects.len() > QT_MAX_OBJECTS && self.depth < QT_MAX_DEPTH {
            self.subdivide();
            let objs = std::mem::take(&mut self.objects);
            if let Some(children) = &mut self.children {
                for (idx, p, r) in objs {
                    for child in children.iter_mut() {
                        child.insert(idx, p, r);
                    }
                }
            }
        }
    }

    fn clear(&mut self) {
        self.objects.clear();
        if let Some(children) = &mut self.children {
            for child in children.iter_mut() {
                child.clear();
            }

            let can_merge = children.iter().all(|c| c.objects.is_empty() && c.children.is_none());
            if can_merge {
                self.children = None;
            }
        }
    }

    fn collect_pairs(&self, pairs: &mut Vec<(usize, usize)>) {
        for i in 0..self.objects.len() {
            for j in (i+1)..self.objects.len() {
                let a = self.objects[i].0;
                let b = self.objects[j].0;

                if a < b {
                    pairs.push((a,b));
                } else {
                    pairs.push((b,a));
                }
            }
        }

        if let Some(children) = &self.children {
            children.iter().for_each(|child| child.collect_pairs(pairs));
        }
    }
}



pub struct QuadTree {
    pub root: QuadTreeNode,
}

impl QuadTree {
    fn new(bounds: Rect) -> Self {
        Self {
            root: QuadTreeNode::new(bounds, 0),
        }
    }

    fn clear(&mut self) {
        self.root.clear();
    }

    fn insert(&mut self, index: usize, pos: Vec2, radius: f32) {
        self.root.insert(index, pos, radius);
    }

    fn collect_pairs(&self, pairs: &mut Vec<(usize, usize)>) {
        pairs.clear();
        self.root.collect_pairs(pairs);
    }

    pub fn average_objects_per_node(&self) -> f32 {
        let (total_objects, total_nodes) = self.count_objects_and_nodes();
        if total_nodes == 0 {
            0.0
        } else {
            total_objects as f32 / total_nodes as f32
        }
    }

    fn count_objects_and_nodes(&self) -> (usize, usize) {
        fn helper(node: &QuadTreeNode) -> (usize, usize) {
            let mut total_objects = node.objects.len();
            let mut total_nodes = 0;

            if let Some(children) = &node.children {
                for child in children.iter() {
                    let (child_objects, child_nodes) = helper(child);
                    total_objects += child_objects;
                    total_nodes += child_nodes;
                }
            } else {
                total_nodes += 1;
            }

            (total_objects, total_nodes)
        }

        helper(&self.root)
    }
}


pub const QT_MAX_DEPTH: usize = 15;
pub const QT_MAX_OBJECTS: usize = 1;




pub struct PhysicsSolver {
    pub objects: Vec<Ball>,
    pub quadtree: QuadTree,
    pub world: Rect,
    pub gravity: Vec2,
    pub sub_steps: u32,
    pairs: Vec<(usize, usize)>,
}

impl PhysicsSolver {
    pub fn new(app: &App, gravity_mag: f32, sub_steps: u32) -> Self {
        let objects: Vec<Ball> = Vec::with_capacity(1000);
        let (w, h) = app.window_rect().w_h();
        let world = Rect::from_corners(pt2(0.0, 0.0), pt2(w, h));

        let quadtree = QuadTree::new(world);
        let pairs = Vec::with_capacity(QT_MAX_DEPTH * QT_MAX_OBJECTS * QT_MAX_OBJECTS);
    
        PhysicsSolver {
            objects,
            quadtree,
            world,
            gravity: vec2(0.0, gravity_mag),
            sub_steps,
            pairs,
        }
    }
    
    pub fn update(&mut self, dt: &f32, window_vel: Vec2) { // main update loop for the full simulation
        let sub_dt = dt / self.sub_steps as f32;
        let sub_window_vel = window_vel / self.sub_steps as f32;

        (0..self.sub_steps).for_each(|_| self.sub_update(&sub_dt, sub_window_vel) );
    }

    fn sub_update(&mut self, dt: &f32, window_vel: Vec2) {
        self.quadtree.clear();

        for (i, obj) in self.objects.iter().enumerate() {
            self.quadtree.insert(i, obj.position, obj.radius);
        }

        self.quadtree.collect_pairs(&mut self.pairs);
        self.solve_collsions();
        self.update_other(&dt, window_vel);
    }

    fn solve_collsions(&mut self) {
        for i in 0..self.pairs.len() {
            let (a, b) = self.pairs[i];
            self.solve_contact(a, b);
        }
    }

    #[inline(always)]
    fn solve_contact(&mut self, b1_ind: usize, b2_ind: usize) { // Single ball on ball collision
        if b1_ind == b2_ind {
            return;
        }
        
        let response_coef = 1.0;
        let eps = 0.0001;
        let eps_sq = eps * eps;
        
        let o2_o1 = self.objects[b1_ind].position - self.objects[b2_ind].position;
        let dist_sq = self.objects[b1_ind].position.distance_squared(self.objects[b2_ind].position);
        let min_dist = self.objects[b1_ind].radius + self.objects[b2_ind].radius;
        let min_dist_sq = min_dist * min_dist;
        
        if dist_sq < min_dist_sq && dist_sq > eps_sq {
            let dist = dist_sq.sqrt();
            let delta = response_coef * 0.5 * (min_dist - dist);
            let col_vec = (o2_o1 / dist) * delta;
            self.objects[b1_ind].position += col_vec;
            self.objects[b2_ind].position -= col_vec;
        }
    }

    fn update_other(&mut self, dt: &f32, window_vel: Vec2) { // other updates for the full simulation
        let gravity = self.gravity;
        let world_w = self.world.w();
        let world_h = self.world.h();

        for (i, obj) in &mut self.objects.iter_mut().enumerate() { 
            obj.add_velocity(window_vel);
            obj.acceleration -= gravity * self.sub_steps as f32;
            obj.update(dt);

            if obj.position.y > world_h - obj.radius {
                obj.last_position.y = world_h - obj.radius;
                obj.position.y = world_h - obj.radius; // - obj.position.
            } else if obj.position.y < obj.radius {
                obj.last_position.y = obj.radius;
                obj.position.y = 2.0 * obj.radius - obj.position.y;
            }

            if obj.position.x > world_w - obj.radius {
                obj.last_position.x = world_w - obj.radius;
                obj.position.x = world_w - obj.radius; // - obj.position.x;
            } else if obj.position.x < obj.radius {
                obj.last_position.x = obj.radius;
                obj.position.x = 2.0 * obj.radius - obj.position.x;
            }
        }
    }
    
    pub fn refresh_quadtree(&mut self) {
        self.quadtree = QuadTree::new(self.world);
    }
    
    pub fn freeze(&mut self) {
        for obj in &mut self.objects {
            obj.last_position = obj.position;
        }
    }

    pub fn add_object(&mut self, obj: Ball) {
        self.objects.push(obj);
    }
}

}
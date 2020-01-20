mod tools;
mod planet;

use ggez::event::{self, KeyCode, KeyMods};
use ggez::graphics::{self, DrawParam, Mesh, MeshBuilder};
use ggez::nalgebra::{Point2, Vector2};
use ggez::{Context, GameResult};
use ggez::timer;
use ggez::input::mouse::MouseButton;

use rand::prelude::*;
use rand::rngs::ThreadRng;

use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use std::time::Duration;
use std::f32::consts::PI;

use planet::{Planet, PlanetTrail};

pub const G: f32 = 0.0001;    // Gravitational constant
pub const TWO_PI: f32 = PI * 2.0;
const SPAWN_PLANET_RADIUS: f32 = 10.0;
const FORCE_DEBUG_VECTOR_MULTIPLIER: f32 = 0.00005;
const SCREEN_DIMS: (f32, f32) = (1280.0, 860.0);

struct MainState {
    planet_id_count: usize,
    planets: HashMap<usize, RefCell<Planet>>,
    planet_trails: HashMap<usize, PlanetTrail>,
    mouse_info: MouseInfo,
    rand_thread: ThreadRng,

    show_vector_debug: bool,
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let mut s = MainState {
            planet_id_count: 0,
            planets: HashMap::new(),
            planet_trails: HashMap::new(),
            mouse_info: MouseInfo::default(),
            rand_thread: rand::thread_rng(),

            show_vector_debug: false,
        };

        // s.add_planet(
        //     Point2::new(640.0, 360.0),
        //     None,
        //     None,
        //     50.0,
        //     None,
        // );

        // const GAP: f32 = 20.0;
        // s.spawn_square_of_planets(
        //     Point2::new(GAP/2.0, GAP/2.0),
        //     (SCREEN_DIMS.0/GAP).ceil() as u16,
        //     (SCREEN_DIMS.1/GAP).ceil() as u16,
        //     GAP,
        //     1.0,
        // );

        // s.add_planet(
        //     Point2::new(750.0, 360.0),
        //     Some(Vector2::new(0.0, 50.0)),
        //     None,
        //     5.0
        // );
        s.restart();

        Ok(s)
    }

    fn restart(&mut self) {
        self.planets = HashMap::new();

        // self.add_planet_with_moons(
        //     Point2::new(640.0, 430.0),
        //     None,
        //     None,
        //     50.0,
        //     700,
        //     (15.0, 100.0),
        //     (0.5, 1.5),
        //     true,
        // );

        self.add_random_planets(
            100,
            (0.0, SCREEN_DIMS.0),
            (0.0, SCREEN_DIMS.1),
            (1.0, 8.0),
            None,
        );
    }

    #[inline]
    fn add_planet(&mut self, position: Point2<f32>, velocity: Option<Vector2<f32>>, mass: Option<f32>, radius: f32, spawn_protection: Option<Duration>) {
        self.add_planet_raw(Planet::new(
            self.planet_id_count,
            position,
            velocity,
            mass,
            radius,
            spawn_protection,
        ));
    }

    // Spawns a planet with other 
    fn add_planet_with_moons(
        &mut self,
        position: Point2<f32>,
        velocity: Option<Vector2<f32>>,
        main_planet_mass: Option<f32>,
        main_planet_radius: f32,
        moon_num: usize,
        moon_orbit_radius_range: (f32, f32),    // Starting from surface of planet
        moon_body_radius_range: (f32, f32),
        orbit_direction_clockwise: bool,  // anticlockwise = false, clockwise = true
    ) {
        self.add_planet(position, velocity, main_planet_mass, main_planet_radius, None);  // Add main planet
        let (main_planet_mass, frame_velocity) = {
            let p = self.planets.get(&(self.planet_id_count - 1)).unwrap().borrow();
            (p.mass, p.velocity)
        };

        for _ in 0..moon_num {
            let orbit_radius = main_planet_radius + self.rand_thread.gen_range(moon_orbit_radius_range.0, moon_orbit_radius_range.1);
            let orbit_speed = tools::circular_orbit_speed(main_planet_mass, orbit_radius);
            let start_angle = self.rand_thread.gen_range(0.0, TWO_PI);      // Angle from main planet to moon
            let start_pos = tools::get_components(orbit_radius, start_angle);   // Position on circle orbit where planet will start
            let start_velocity = tools::get_components(
                orbit_speed,
                if orbit_direction_clockwise {
                    start_angle + PI/2.0
                } else {
                    start_angle - PI/2.0
                }
            );  // 90 degrees to angle with planet
            let moon_radius = self.rand_thread.gen_range(moon_body_radius_range.0, moon_body_radius_range.1);

            self.add_planet(
                position + start_pos,
                Some(start_velocity + frame_velocity),  // Add velocity of main planet
                None,
                moon_radius,
                None,
            );
        }
    }

    #[inline]
    fn add_planet_raw(&mut self, mut planet: Planet) {
        planet.id = self.planet_id_count;

        self.planet_trails.insert(
            self.planet_id_count,
            PlanetTrail::new(planet.position)
        );

        self.planets.insert(
            self.planet_id_count,
            RefCell::new(planet)
        );

        self.planet_id_count += 1;
    }

    #[inline]
    fn add_random_planets(&mut self, n: usize, x_range: (f32, f32), y_range: (f32, f32), radius_range: (f32, f32), speed_range: Option<(f32, f32)>) {
        assert!(x_range.1 > x_range.0);
        assert!(y_range.1 > y_range.0);
        assert!(radius_range.1 > radius_range.0);
        assert!(n > 0);

        for _ in 0..n {
            let x_pos = self.rand_thread.gen_range(x_range.0, x_range.1);
            let y_pos = self.rand_thread.gen_range(y_range.0, y_range.1);
            let radius = self.rand_thread.gen_range(radius_range.0, radius_range.1);

            let velocity = if let Some(speed_range) = speed_range {
                assert!(speed_range.1 > speed_range.0);

                let speed = self.rand_thread.gen_range(speed_range.0, speed_range.1);
                let angle = self.rand_thread.gen_range(0.0, TWO_PI);
                Some(tools::get_components(speed, angle))
            } else {
                None
            };

            self.add_planet(
                Point2::new(x_pos, y_pos),
                velocity,
                None,
                radius,
                None,
            );
        }
    }

    #[inline]
    fn remove_planet(&mut self, id: usize) {
        if self.planets.remove(&id).is_none() {
            println!("WARNING: Tried to remove planet {} but it wasn't in the hashmap.", id);
        }
    }

    #[inline]
    fn draw_debug_info(&self, ctx: &mut Context) -> GameResult {
        let text = graphics::Text::new(
            format!(
                "{:.3}\nBodies: {}\nPlanet Trails: {}\nTrail Node Count: {}",
                timer::fps(ctx),
                self.planets.len(),
                self.planet_trails.len(),
                self.node_count(),
            )
        );
        graphics::draw(
            ctx,
            &text,
            DrawParam::new().dest([10.0, 10.0])
        )
    }

    fn draw_vectors(&self, ctx: &mut Context, velocities: bool, forces: bool) -> GameResult {
        for (_, planet) in self.planets.iter() {
            let planet_borrow = planet.borrow();
            if velocities && planet_borrow.velocity.magnitude_squared() > 1.0 {
                let vel_line = graphics::Mesh::new_line(
                    ctx,
                    &[planet_borrow.position, planet_borrow.position + planet_borrow.velocity],
                    1.0,
                    [0.0, 1.0, 0.0, 1.0].into()
                )?;
                graphics::draw(ctx, &vel_line, DrawParam::default())?;
            }

            if forces && planet_borrow.resultant_force.magnitude_squared() > 1.0/FORCE_DEBUG_VECTOR_MULTIPLIER {
                let force_line = graphics::Mesh::new_line(
                    ctx,
                    &[planet_borrow.position, planet_borrow.position + planet_borrow.resultant_force * FORCE_DEBUG_VECTOR_MULTIPLIER],
                    1.0,
                    [1.0, 0.0, 0.0, 1.0].into()
                )?;
                graphics::draw(ctx, &force_line, DrawParam::default())?;
            }
        }
        Ok(())
    }

    pub fn draw_mouse_drag(ctx: &mut Context, mouse_info: &MouseInfo) -> GameResult {
        let line = Mesh::new_line(
            ctx,
            &[mouse_info.down_pos, mouse_info.current_drag_position],
            2.0,
            [0.0, 1.0, 0.0, 1.0].into(),
        )?;
        graphics::draw(ctx, &line, DrawParam::default())?;
        tools::draw_circle(ctx, mouse_info.down_pos, SPAWN_PLANET_RADIUS, [1.0, 1.0, 1.0, 0.4].into())?;

        Ok(())
    }

        #[inline]
    fn collide_planets(&mut self, planets: &HashSet<usize>) -> Planet {  // Returns new planet that is sum of other two.
        // Conservation of momentum
        let mut total_mass = 0.0;
        let mut total_volume = 0.0;
        let mut total_momentum = Vector2::new(0.0, 0.0);
        let mut sum_of_rm = Point2::new(0.0, 0.0);      // Centre of mass of system is this divided by total mass of all bodies

        for id in planets.iter().filter(|id| self.planets.contains_key(id)) {
            let p = self.planets.get(id).expect(&format!("Planet {} not in hashmap.", id)).borrow();
            total_mass += p.mass;
            total_volume += tools::volume_of_sphere(p.radius);
            total_momentum += p.mass * p.velocity;

            sum_of_rm.x += p.position.x * p.mass;
            sum_of_rm.y += p.position.y * p.mass;
        }

        let new_radius = tools::inverse_volume_of_sphere(total_volume);
        // Use centre of mass as new position
        let new_position = sum_of_rm/total_mass;

        // Connect ends of trails
        for (_, trail) in self.planet_trails.iter_mut().filter(|(id, _)| planets.contains(id)) {
            trail.add_node(new_position);
        }

        // ID is set to 0, and is then changed afterwards.
        Planet::new(0, new_position, Some(total_momentum/total_mass), Some(total_mass), new_radius, None)
    }

    fn spawn_square_of_planets(
        &mut self,
        top_left: Point2<f32>,
        w: u16,
        h: u16,
        gap: f32,
        rad: f32,
    ) {
        for i in 0..w {
            for j in 0..h {
                self.add_planet(
                    Point2::new(top_left.x + i as f32 * gap, top_left.y + j as f32 * gap),
                    None,
                    None,
                    rad,
                    None,
                );
            }
        }
    }

    fn update_planet_trails(&mut self, dt_duration: &Duration) {
        for (id, trail) in self.planet_trails.iter_mut() {
            trail.update(
                dt_duration,
                if let Some(planet) = self.planets.get(&id) {
                    Some(planet.borrow().position)
                } else {
                    None
                },
            );
        }
    }

    fn node_count(&self) -> usize {
        let mut total = 0;
        for (_, trail) in self.planet_trails.iter() {
            total += trail.node_count();
        }

        total
    }

    #[inline]
    fn put_in_collision_group(collision_groups: &mut Vec<HashSet<usize>>, i_id: usize, j_id: usize) {
        let mut now_in_group = false;
        for collision_group in collision_groups.iter_mut() {
            let contains_i = collision_group.contains(&i_id);
            let contains_j = collision_group.contains(&j_id);

            if contains_i && contains_j {
                // Do nothing
            } else if contains_i {
                collision_group.insert(j_id);
            } else if contains_j {
                collision_group.insert(i_id);
            }

            if contains_i || contains_j {
                now_in_group = true;
                break
            }
        }

        if !now_in_group {  // Start a new group
            let mut new_set = HashSet::with_capacity(2);
            new_set.insert(i_id);
            new_set.insert(j_id);
            collision_groups.push(new_set);
        }
    }

    #[inline]
    fn resolve_collisions(&mut self, collision_groups: &Vec<HashSet<usize>>) {
        let mut new_planets = Vec::new();
        for collision_group in collision_groups.iter() {
            new_planets.push(self.collide_planets(&collision_group));
            // Remove planets in each collision group (since they will be replaced by new planet)
            for id in collision_group {
                self.remove_planet(*id);
            }
        }

        // Add new planets
        for planet in new_planets {
            //self.debris_emitters.push(ParticleSystem::new(planet.position, ParticleSystemParam::debris_emitter()));
            self.add_planet_raw(planet);
        }
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let dt_duration = timer::delta(ctx);
        let dt = timer::duration_to_f64(dt_duration) as f32;

        /*
            Groups that are colliding.
            E.g: vec![ {1, 4, 2}, {5, 3} ]
        */
        let mut collision_groups: Vec<HashSet<usize>> = Vec::with_capacity(self.planets.len()/2);

        // Remove dead particle emitters
        self.planet_trails.retain(|_, trail| !trail.is_dead());

        let keys: Vec<&usize> = self.planets.keys().collect();
        let len = self.planets.len();

        if len > 0 {
            // Update planets
            for (_, pl) in self.planets.iter() {
                pl.borrow_mut().update(dt, &dt_duration);
            }

            for i in 0..len-1 {
                let pl1 = self.planets.get(keys[i]).expect("Couldn't get planet 1");
                for j in i+1..len {
                    let pl2 = self.planets.get(keys[j]).expect("Couldn't get planet 2");
                    let (colliding, protection) = {
                        let bpl1 = pl1.borrow();
                        let bpl2 = pl2.borrow();
                        (tools::check_collision(&bpl1, &bpl2), bpl1.has_spawn_protection() || bpl2.has_spawn_protection())
                    };
    
                    // Check for collision even if they have spawn protection, since I do not want to apply grav
                    // force when planets are inside of each other (as they become very speedy).
                    // protection is true if either planets have spawn protection
                    if colliding && !protection {
                        Self::put_in_collision_group(&mut collision_groups, *keys[i], *keys[j]);
                    } else if !colliding {
                        tools::newtonian_grav(&mut pl1.borrow_mut(), &mut pl2.borrow_mut());
                    }
                }
            }

            self.resolve_collisions(&collision_groups);
        }

        // Update trails
        self.update_planet_trails(&dt_duration);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());

        if self.mouse_info.down && self.mouse_info.button_down == MouseButton::Left &&
            (self.mouse_info.down_pos.x - self.mouse_info.current_drag_position.x).powi(2) +
            (self.mouse_info.down_pos.y - self.mouse_info.current_drag_position.y).powi(2) >= 4.0
        {
            Self::draw_mouse_drag(ctx, &self.mouse_info)?;
            //self.draw_fake_planet(ctx, self.mouse_info.down_pos, 5.0)?;
        }

        // Draw particles
        {
            let mut lines_mesh_builder = MeshBuilder::new();
            let mut are_lines = false;
    
            for (_, trail) in self.planet_trails.iter() {
                trail.draw(&mut lines_mesh_builder)?;
                if !are_lines && trail.node_count() > 1 {
                    are_lines = true;
                }
            }
            
            if are_lines {     // Prevents lyon error when building mesh
                let line_mesh = lines_mesh_builder.build(ctx)?;
                graphics::draw(ctx, &line_mesh, DrawParam::default())?;
            }
        }


        // Draw planets on top of particles
        if !self.planets.is_empty() {
            let mut planets_mesh_builder = MeshBuilder::new();

            for (_, planet) in self.planets.iter() {
                planet.borrow().draw(&mut planets_mesh_builder);
            }
    
            let planets_mesh = planets_mesh_builder.build(ctx)?;
            graphics::draw(ctx, &planets_mesh, DrawParam::default())?;
        }

        if self.show_vector_debug {
            self.draw_vectors(ctx, true, true)?;
        }

        self.draw_debug_info(ctx)?;
        graphics::present(ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        self.mouse_info.down = true;
        self.mouse_info.button_down = button;
        self.mouse_info.down_pos = Point2::new(x, y);
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        self.mouse_info.down = false;

        if button == MouseButton::Left {
            self.add_planet(
                self.mouse_info.down_pos,
                Some(self.mouse_info.down_pos - Point2::new(x, y)),
                None,
                SPAWN_PLANET_RADIUS,
                None,
            );
        }
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.mouse_info.current_drag_position = Point2::new(x, y);
    }


    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::D => self.show_vector_debug = !self.show_vector_debug,
            KeyCode::R => self.restart(),
            _ => (),
        }
    }
}


struct MouseInfo {
    down: bool,
    button_down: MouseButton,
    down_pos: Point2<f32>,
    current_drag_position: Point2<f32>,
}

impl Default for MouseInfo {
    fn default() -> MouseInfo {
        MouseInfo {
            down: false,
            button_down: MouseButton::Left,
            down_pos: Point2::new(0.0, 0.0),
            current_drag_position: Point2::new(1.0, 0.0),
        }
    }
}

pub fn main() -> GameResult {
    use std::path;
    use std::env;
    use ggez::conf::{WindowMode, WindowSetup, NumSamples};

    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("Planets", "ggez")
        .add_resource_path(resource_dir)
        .window_mode(
            WindowMode::default()
                .dimensions(SCREEN_DIMS.0, SCREEN_DIMS.1)
        )
        .window_setup(
            WindowSetup::default()
                .samples(NumSamples::Four)
        );

    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}

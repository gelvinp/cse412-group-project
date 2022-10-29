use postgres::{Client, NoTls, Config};
use r2d2_postgres::PostgresConnectionManager;
use r2d2::{Pool};
use gdnative::prelude::*;
use std::str::FromStr;

#[derive(Clone)]
pub struct PGConnection
{
    pub pool: Option<Pool<PostgresConnectionManager<NoTls>>>,
    role: String,
}

impl PGConnection
{
    pub fn new() -> Self { Self { pool: None, role: String::new() } }

    pub fn connected(&self) -> bool
    {
        self.pool.is_some()
    }

    pub fn connect(&mut self, ip: &str, port: &str, name: &str, user: &str, pass: &str) -> bool
    {
        self.role = user.to_owned();

        let url = format!("postgresql://{}:{}@{}:{}/{}", user, pass, ip, port, name);

        let manager = match Config::from_str(&url)
        {
            Ok(config) =>
            {
                PostgresConnectionManager::new(config, NoTls)
            }

            Err(err) =>
            {
                godot_print!("{}", err);
                return false;
            }
        };

        self.pool = match r2d2::Pool::builder()
            .max_size(20)
            .min_idle(Some(1))
            .build(manager)
        {
            Ok(pool) => Some(pool),
            Err(err) =>
            {
                godot_print!("{}", err);
                return false;
            }
        };

        godot_print!("Got {} connections", self.pool.as_ref().unwrap().state().connections);

        true
    }

    pub fn max_connections(&self) -> u32
    {
        self.pool.as_ref().and_then(|p| Some(p.state().connections)).unwrap_or(0)
    }

    pub fn init(&mut self) -> bool
    {
        let pool = match &mut self.pool
        {
            Some(pool) => pool,
            None => { return false; },
        };

        let mut client = match pool.get()
        {
            Ok(client) => client,
            Err(err) =>
            {
                godot_print!("{}", err);
                return false;
            }
        };

        match client.batch_execute(&format!("
            DROP SCHEMA public CASCADE;
            CREATE SCHEMA public;
            GRANT ALL ON SCHEMA public TO {}, public;

            CREATE TABLE regions (
                r_region_id int PRIMARY KEY,
                r_coord_x smallint,
                r_coord_y smallint
            );

            CREATE INDEX idx_r_region_id ON regions(r_region_id);

            CREATE TABLE timepoints (
                t_timepoint_id smallint PRIMARY KEY,
                t_year smallint,
                t_month smallint
            );

            CREATE INDEX idx_t_timepoint_id ON timepoints(t_timepoint_id);

            CREATE TABLE countries (
                c_iso_a3 char(3) PRIMARY KEY,
                c_name varchar(128),
                c_center_x smallint,
                c_center_y smallint
            );

            CREATE INDEX idx_c_iso_a3 ON countries(c_iso_a3);

            CREATE TABLE region_in_country (
                ric_region_id int,
                ric_iso_a3 char(3),
                PRIMARY KEY (ric_region_id, ric_iso_a3),
                CONSTRAINT fk_region FOREIGN KEY(ric_region_id) REFERENCES regions(r_region_id),
                CONSTRAINT fk_country FOREIGN KEY(ric_iso_a3) REFERENCES countries(c_iso_a3)
            );

            CREATE INDEX idx_ric ON region_in_country(ric_region_id, ric_iso_a3);

            CREATE UNLOGGED TABLE weatherpoints (
                wp_region_id int,
                wp_timepoint_id smallint,
                wp_prec smallint,
                wp_tmin smallint,
                wp_tmax smallint,
                PRIMARY KEY (wp_region_id, wp_timepoint_id),
                CONSTRAINT fk_region FOREIGN KEY(wp_region_id) REFERENCES regions(r_region_id),
                CONSTRAINT fk_timepoint FOREIGN KEY(wp_timepoint_id) REFERENCES timepoints(t_timepoint_id)
            );

            CREATE INDEX idx_wp ON weatherpoints(wp_region_id, wp_timepoint_id);
        ", &self.role))
        {
            Ok(_) => true,
            Err(err) =>
            {
                godot_print!("{}", err);
                false
            }
        }
    }

    pub fn insert_regions(&mut self, regions: &[String]) -> bool
    {
        let pool = match &mut self.pool
        {
            Some(pool) => pool,
            None => { return false; },
        };

        let mut client = match pool.get()
        {
            Ok(client) => client,
            Err(err) =>
            {
                godot_print!("{}", err);
                return false;
            }
        };

        match client.batch_execute(&format!("
            INSERT INTO regions VALUES {};
        ", regions.join(",")))
        {
            Ok(_) => true,
            Err(err) =>
            {
                godot_print!("{}", err);
                false
            }
        }
    }

    pub fn insert_timepoints(&mut self, timepoints: &[String]) -> bool
    {
        let pool = match &mut self.pool
        {
            Some(pool) => pool,
            None => { return false; },
        };

        let mut client = match pool.get()
        {
            Ok(client) => client,
            Err(err) =>
            {
                godot_print!("{}", err);
                return false;
            }
        };

        match client.batch_execute(&format!("
            INSERT INTO timepoints VALUES {};
        ", timepoints.join(",")))
        {
            Ok(_) => true,
            Err(err) =>
            {
                godot_print!("{}", err);
                false
            }
        }
    }

    pub fn insert_weatherpoints(&mut self, weatherpoints: &[String]) -> bool
    {
        let pool = match &mut self.pool
        {
            Some(pool) => pool,
            None => { return false; },
        };

        let mut client = match pool.get()
        {
            Ok(client) => client,
            Err(err) =>
            {
                godot_print!("{}", err);
                return false;
            }
        };

        match client.batch_execute(&format!("
            INSERT INTO weatherpoints VALUES {};
        ", weatherpoints.join(",")))
        {
            Ok(_) => true,
            Err(err) =>
            {
                godot_print!("{}", err);
                false
            }
        }
    }

    pub fn insert_country(&mut self, iso_a3: &str, name: &str, x: i16, y: i16, regions: &Vec<u32>) -> bool
    {
        let pool = match &mut self.pool
        {
            Some(pool) => pool,
            None => { return false; },
        };

        let mut client = match pool.get()
        {
            Ok(client) => client,
            Err(err) =>
            {
                godot_print!("{}", err);
                return false;
            }
        };

        match client.execute("
            INSERT INTO countries VALUES ($1, $2, $3, $4);
        ", &[&iso_a3, &name, &x, &y])
        {
            Ok(res) =>
            {
                if res != 1
                {
                    return false;
                }
            }
            Err(err) =>
            {
                godot_print!("{}", err);
                return false;
            }
        };

        for region in regions
        {
            let region_signed = match i32::try_from(*region)
            {
                Ok(signed) => signed,
                Err(err) =>
                {
                    godot_print!("Could not convert to signed: {}", err);
                    return false;
                }
            };
            
            match client.execute("
                INSERT INTO region_in_country VALUES ($1, $2);
            ", &[&region_signed, &iso_a3])
            {
                Ok(res) =>
                {
                    if res != 1
                    {
                        return false;
                    }
                }
                Err(err) =>
                {
                    godot_print!("{}", err);
                    return false;
                }
            };
        }

        true
    }
}
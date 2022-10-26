use postgres::{Client, NoTls, Config};
use r2d2_postgres::PostgresConnectionManager;
use r2d2::{Pool};
use gdnative::prelude::*;
use std::str::FromStr;

pub struct PGConnection
{
    pub connection: Option<Client>,
    pub pool: Option<Pool<PostgresConnectionManager<NoTls>>>,
    role: String,
}

impl PGConnection
{
    pub fn new() -> Self { Self { connection: None, pool: None, role: String::new() } }

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
            .max_size(15)
            .build(manager)
        {
            Ok(pool) => Some(pool),
            Err(err) =>
            {
                godot_print!("{}", err);
                return false;
            }
        };

        match Client::connect(&url, NoTls)
        {
            Ok(conn) =>
            {
                self.connection = Some(conn);
                true
            }

            Err(_) => false
        }
    }

    pub fn init(&mut self) -> bool
    {
        let client = match &mut self.connection
        {
            Some(client) => client,
            None => { return false; },
        };

        match client.batch_execute(&format!("
            DROP SCHEMA public CASCADE;
            CREATE SCHEMA public;
            GRANT ALL ON SCHEMA public TO {}, public;

            CREATE TABLE \"Regions\" (
                r_region_id int PRIMARY KEY,
                r_coord_x smallint,
                r_coord_y smallint
            );

            CREATE TABLE \"Timepoints\" (
                t_timepoint_id smallint PRIMARY KEY,
                t_year smallint,
                t_month smallint
            );

            CREATE TABLE \"Countries\" (
                c_iso_a3 char(3) PRIMARY KEY,
                c_name varchar(128),
                c_center_x smallint,
                c_center_y smallint
            );

            CREATE TABLE \"RegionInCountry\" (
                ric_region_id int,
                ric_iso_a3 char(3),
                PRIMARY KEY (ric_region_id, ric_iso_a3),
                CONSTRAINT fk_region FOREIGN KEY(ric_region_id) REFERENCES \"Regions\"(r_region_id),
                CONSTRAINT fk_country FOREIGN KEY(ric_iso_a3) REFERENCES \"Countries\"(c_iso_a3)
            );

            CREATE TABLE \"Weatherpoints\" (
                wp_region_id int,
                wp_timepoint_id smallint,
                wp_prec smallint,
                wp_tmin smallint,
                wp_tmax smallint,
                PRIMARY KEY (wp_region_id, wp_timepoint_id),
                CONSTRAINT fk_region FOREIGN KEY(wp_region_id) REFERENCES \"Regions\"(r_region_id),
                CONSTRAINT fk_timepoint FOREIGN KEY(wp_timepoint_id) REFERENCES \"Timepoints\"(t_timepoint_id)
            );
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
        let client = match &mut self.connection
        {
            Some(client) => client,
            None => { return false; },
        };

        match client.batch_execute(&format!("
            INSERT INTO \"Regions\" VALUES {};
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
        let client = match &mut self.connection
        {
            Some(client) => client,
            None => { return false; },
        };

        match client.batch_execute(&format!("
            INSERT INTO \"Timepoints\" VALUES {};
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
        let client = match &mut self.connection
        {
            Some(client) => client,
            None => { return false; },
        };

        match client.batch_execute(&format!("
            INSERT INTO \"Weatherpoints\" VALUES {};
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

    pub fn insert_country(&mut self, iso_a3: &str, name: &str, x: u32, y: u32, regions: &[u32]) -> bool
    {
        let client = match &mut self.connection
        {
            Some(client) => client,
            None => { return false; },
        };

        match client.execute("
            INSERT INTO \"Countries\" VALUES ($1, $2, $3, $4);
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
            match client.execute("
                INSERT INTO \"RegionInCountry\" VALUES ($1, $2);
            ", &[&region, &iso_a3])
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
use postgres::{Client, NoTls};

pub struct PGConnection
{
    pub connection: Option<Client>,
}

impl PGConnection
{
    pub fn new() -> Self { Self { connection: None::<Client> } }

    pub fn connected(&self) -> bool
    {
        self.connection.is_some()
    }

    pub fn connect(&mut self, ip: &str, port: &str, name: &str, user: &str, pass: &str) -> bool
    {
        let url = format!("postgresql://{}:{}@{}:{}/{}", user, pass, ip, port, name);
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

    pub fn clean(&mut self) -> bool
    {
        let client = match &mut self.connection
        {
            Some(client) => client,
            None => { return false; },
        };

        client.batch_execute("
            DROP SCHEMA public CASCASDE;
            CREATE SCHEMA public;
            GRANT ALL ON SCHEMA public TO postgres, public;

            CREATE TABLE Regions (
                r_region_id int PRIMARY KEY,
                r_coord_x int,
                r_coord_y int,
            );

            CREATE TABLE Timepoints (
                t_timepoint_id int PRIMARY KEY,
                t_year int,
                t_month int,
            );

            CREATE TABLE Countries (
                c_iso_a3 char(3) PRIMARY KEY,
                c_coord_x int,
                c_coord_y int,
                c_name varchar(128)
            );

            CREATE TABLE RegionInCountry (
                ric_region_id int,
                ric_iso_a3 int,
                PRIMARY KEY (ric_region_id, ric_iso_a3)
                CONSTRAINT fk_region FOREIGN_KEY(ric_region_id) REFERENCES Regions(r_region_id)
                CONSTRAINT fk_country FOREIGN_KEY(ric_iso_a3) REFERENCES Countries(c_iso_a3)
            );

            CREATE TABLE WeatherPoint (
                wp_region_id int,
                wp_timepoint_id int,
                wp_prec int,
                wp_tmin int,
                wp_tmax int,
                PRIMARY KEY (wp_region_id, wp_country_id)
                CONSTRAINT fk_region FOREIGN_KEY(wp_region_id) REFERENCES Regions(r_region_id)
                CONSTRAINT fk_timepoint FOREIGN_KEY(wp_timepoint_id) REFERENCES Timepoints(t_timepoint_id)
            );
        ").is_ok()
    }
}
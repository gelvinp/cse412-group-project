mod connection;


use gdnative::prelude::*;
use gdnative::export::user_data::MutexData;
use connection::PGConnection;
use std::hash::Hash;
use std::sync::{Mutex, Arc};
use std::collections::HashMap;
use gdnative::api::{Image, ImageTexture};


pub struct RegionCoord
{
    pub x: i32,
    pub y: i32
}


#[derive(NativeClass)]
#[inherit(Node)]
pub struct DatabaseConnection
{
    connection: PGConnection,
    region_coords: HashMap<i32, RegionCoord>,
}


impl DatabaseConnection
{
	fn new(_base: &Node) -> Self
    {
        DatabaseConnection
        {
            connection: PGConnection::new(),
            region_coords: HashMap::<i32, RegionCoord>::new(),
        }
    }
}

#[methods]
impl DatabaseConnection
{
    #[method]
    fn db_connect(&mut self, ip: String, port: String, name: String, user: String, pass: String) -> bool
    {
        if self.connection.connect(&ip, &port, &name, &user, &pass)
        {
            godot_print!("Connected, fetching regions");

            if let Some(region_coords) = self.connection.get_regions()
            {
                self.region_coords = region_coords;
                return true;
            }
            else
            {
                godot_print!("Failed to get regions");
            }
        }

        return false;
    }

    #[method]
    fn get_countries(&mut self) -> HashMap<String, (i32, i32, String)>
    {
        self.connection.get_countries().unwrap_or_default()
    }

    #[method]
    fn get_data_for_timepoint(&mut self, timepoint_id: i32) -> Vec<(i32, i32, i32)>
    {
        self.connection.get_data_for_timepoint(timepoint_id).unwrap_or_default()
    }

    #[method]
    // field must be one of "wp_prec" "wp_tmin" "wp_tmax"
    fn get_texture_for_timepoint(&mut self, timepoint_id: i16, field: String) -> Ref<ImageTexture, Unique>
    {
        let img = Image::new();
        img.create(8640, 4320, false, Image::FORMAT_L8);
        img.fill(Color::from_rgb(0.0, 0.0, 0.0));

        img.lock();

        if let Some(data) = self.connection.get_data_for_timepoint_field(timepoint_id, &field)
        {
            for point in data
            {
                if let Some(region) = self.region_coords.get(&point.0)
                {
                    img.set_pixel(region.x as i64, region.y as i64, Color::from_hsv(0.0, 0.0, point.1 as f32 / 256.0));
                }
            }
        }

        img.unlock();

        let tex = ImageTexture::new();
        tex.create_from_image(img, 7);

        tex
    }

    #[method]
    // field must be one of "wp_prec" "wp_tmin" "wp_tmax"
    fn get_texture_for_timepoint_country(&mut self, timepoint_id: i16, field: String, iso_a3: String) -> Ref<ImageTexture, Unique>
    {
        let img = Image::new();
        img.create(8640, 4320, false, Image::FORMAT_L8);
        img.fill(Color::from_rgb(0.0, 0.0, 0.0));

        img.lock();

        if let Some(data) = self.connection.get_data_for_timepoint_field_country(timepoint_id, &field, &iso_a3)
        {
            for point in data
            {
                if let Some(region) = self.region_coords.get(&point.0)
                {
                    img.set_pixel(region.x as i64, region.y as i64, Color::from_hsv(0.0, 0.0, point.1 as f32 / 256.0));
                }
            }
        }

        img.unlock();

        let tex = ImageTexture::new();
        tex.create_from_image(img, 7);

        tex
    }
}
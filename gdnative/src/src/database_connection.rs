mod connection;


use gdnative::prelude::*;
use gdnative::export::user_data::MutexData;
use connection::PGConnection;
use std::sync::{Mutex, Arc};
use std::collections::HashMap;


#[derive(NativeClass)]
#[inherit(Node)]
pub struct DatabaseConnection
{
    connection: PGConnection,
}


impl DatabaseConnection
{
	fn new(_base: &Node) -> Self
    {
        DatabaseConnection
        {
            connection: PGConnection::new(),
        }
    }
}

#[methods]
impl DatabaseConnection
{
    #[method]
    fn db_connect(&mut self, ip: String, port: String, name: String, user: String, pass: String) -> bool
    {
        self.connection.connect(&ip, &port, &name, &user, &pass)
    }

    #[method]
    fn get_countries(&mut self) -> HashMap<String, (i32, i32)>
    {
        self.connection.get_countries().unwrap_or_default()
    }
}
mod status;
mod connection;


use gdnative::prelude::*;
use std::borrow::Borrow;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, mpsc::{Receiver, sync_channel, channel}};
use std::time::Duration;
use status::Status;
use gdnative::export::user_data::MutexData;
use std::io::{BufReader, Error, Read, Result, ErrorKind, BufWriter, Write};
use std::fs::{File, self};
use std::thread;
use connection::PGConnection;


const BATCH_SIZE: usize = 100_000;


#[derive(Clone, Copy)]
struct Timepoint(u16, u16);


#[derive(NativeClass)]
#[inherit(Node)]
#[user_data(MutexData<DatabaseInitializer>)]
pub struct DatabaseInitializer
{
    status: Arc<Mutex<Status>>,
    connection: Arc<Mutex<PGConnection>>,
}


impl DatabaseInitializer
{
	fn new(_base: &Node) -> Self
    {
        DatabaseInitializer
        {
            status: Arc::new(Mutex::new(Status::new())),
            connection: Arc::new(Mutex::new(PGConnection::new())),
        }
    }
}

#[methods]
impl DatabaseInitializer
{
    #[method]
    fn connect(&mut self, #[base] owner: &Node, ip: String, port: String, name: String, user: String, pass: String) -> bool
    {
        match self.connection.lock()
        {
            Ok(mut res) => res.connect(&ip, &port, &name, &user, &pass),
            Err(_) => false
        }
    }

    #[method]
	fn init_db(&mut self, #[base] owner: &Node)
	{
        let thread_state = Arc::clone(&self.status);
        let thread_connection = Arc::clone(&self.connection);

        match self.connection.lock()
        {
            Ok(mut res) => 
            {
                if !res.init()
                {
                    set_error(&thread_state, Error::new(ErrorKind::Other, "Failed to create database schema"));
                    return;
                }
            }
            
            Err(_) =>
            {
                set_error(&thread_state, Error::new(ErrorKind::Other, "Failed to create database schema"));
                return;
            }
        };

        thread::spawn(move||
        {
            let region_count = match import_regions(&thread_state, &thread_connection)
            {
                Ok(count) => count,
                Err(err) =>
                {
                    set_error(&thread_state, err);
                    return;
                }
            };

            let timepoints = match import_timepoints(&thread_state, &thread_connection)
            {
                Ok(timepoints) => timepoints,
                Err(err) =>
                {
                    set_error(&thread_state, err);
                    return;
                }
            };

            match import_weatherpoints(&thread_state, &thread_connection, timepoints, region_count)
            {
                Ok(_) => {},
                Err(err) =>
                {
                    set_error(&thread_state, err);
                    return;
                }
            };

            match import_countries(&thread_state, &thread_connection)
            {
                Ok(_) => {},
                Err(err) =>
                {
                    set_error(&thread_state, err);
                    return;
                }
            };

            match thread_state.lock()
            {
                Ok(mut res) => res.completed = true,
                Err(_) => {}
            };
        });
	}

    #[method]
    fn get_status(&mut self, #[base] owner: &Node) -> Dictionary<Unique>
    {
        match self.status.lock()
        {
            Ok(res) =>
            {
                res.dictionary()
            }

            Err(e) =>
            {
                owner.emit_signal("error", &[Variant::new(format!("Failed to aquire the status mutex: {}", e))]);
                
                Dictionary::new()
            }
        }
    }

    #[method]
    fn cancel(&mut self, #[base] owner: &Node)
    {
        godot_print!("Cancelled");
        match self.status.lock()
        {
            Ok(mut res) =>
            {
                res.cancelled = true;
            }

            Err(e) =>
            {
                owner.emit_signal("error", &[Variant::new(format!("Failed to aquire the status mutex: {}", e))]);
            }
        }
    }
}

fn import_regions(status: &Arc<Mutex<Status>>, connection: &Arc<Mutex<PGConnection>>) -> Result<u32>
{
    godot_print!("Importing Regions");

    let mut file = BufReader::new(File::open("large_data/regions/regions.bin")?);

    let mut header = [0u8; 1];
    file.read(&mut header)?;

    if header[0] != 0xAA
    {
        return Err(Error::new(ErrorKind::InvalidData, "Regions File Invalid"));
    }

    let mut countBuffer = [0u8; 4];
    file.read(&mut countBuffer)?;
    
    let region_count = u32::from_le_bytes(countBuffer);

    match status.lock()
    {
        Ok(mut res) =>
        {
            if res.cancelled
            {
                return Ok(region_count)
            }

            res.set_stage(1);
            res.current_progress = 0;
            res.total_work = region_count;
            res.discrete_progress = true;
        }

        Err(e) =>
        {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    }

    let mut regions = Vec::<String>::with_capacity(region_count as usize);

    for index in 0..region_count
    {
        let mut xBuffer = [0u8; 2];
        let mut yBuffer = [0u8; 2];
        file.read_exact(&mut xBuffer)?;
        file.read_exact(&mut yBuffer)?;

        let x = u16::from_le_bytes(xBuffer);
        let y = u16::from_le_bytes(yBuffer);

        regions.push(format!("({}, {}, {})", index, x, y));

        match status.lock()
        {
            Ok(mut res) =>
            {
                if res.cancelled
                {
                    return Ok(region_count)
                }
                
                res.current_progress = index + 1;
            }

            Err(e) =>
            {
                return Err(Error::new(ErrorKind::Other, e.to_string()));
            }
        }
    }

    let batches = ((region_count as usize / BATCH_SIZE) + 1) as usize;

    match status.lock()
    {
        Ok(mut res) =>
        {
            if res.cancelled
            {
                return Ok(region_count)
            }

            res.set_stage(2);
            res.current_progress = 0;
            res.total_work = batches as u32;
            res.discrete_progress = true;
        }

        Err(e) =>
        {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    }

    for batch in 0..batches
    {
        let batch_start = batch * BATCH_SIZE;
        let batch_end = std::cmp::min(batch_start + BATCH_SIZE, region_count as usize);

        match connection.lock()
        {
            Ok(mut res) =>
            {
                if !res.insert_regions(&regions[batch_start..batch_end])
                {
                    return Err(Error::new(ErrorKind::Other, "Failed to insert regions"));
                }
            }
    
            Err(e) =>
            {
                return Err(Error::new(ErrorKind::Other, e.to_string()));
            }
        }

        match status.lock()
        {
            Ok(mut res) =>
            {
                if res.cancelled
                {
                    return Ok(region_count)
                }
                
                res.current_progress = batch as u32 + 1;
            }

            Err(e) =>
            {
                return Err(Error::new(ErrorKind::Other, e.to_string()));
            }
        }
    }

    Ok(region_count)
}

fn import_timepoints(status: &Arc<Mutex<Status>>, connection: &Arc<Mutex<PGConnection>>) -> Result<Vec<Timepoint>>
{
    godot_print!("Importing Timepoints");

    let mut file = BufReader::new(File::open("large_data/timepoints/timepoints.bin")?);

    let mut header = [0u8; 1];
    file.read(&mut header)?;

    if header[0] != 0xBB
    {
        return Err(Error::new(ErrorKind::InvalidData, "Timepoints File Invalid"));
    }

    let mut countBuffer = [0u8; 4];
    file.read(&mut countBuffer)?;
    
    let count = u32::from_le_bytes(countBuffer);

    let mut timepoints = Vec::<Timepoint>::with_capacity(count as usize);
    let mut timepoint_strings = Vec::<String>::with_capacity(count as usize);

    match status.lock()
    {
        Ok(mut res) =>
        {
            if res.cancelled
            {
                return Ok(timepoints)
            }

            res.set_stage(3);
            res.current_progress = 0;
            res.total_work = count;
            res.discrete_progress = true;
        }

        Err(e) =>
        {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    }

    for index in 0..count
    {
        let mut timeBuffer = [0u8; 2];
        file.read(&mut timeBuffer)?;

        let encoded = u16::from_le_bytes(timeBuffer);

        let year = (encoded & 0xFFF0) >> 4;
        let month = encoded & 0x000F;

        timepoints.push(Timepoint(year, month));
        timepoint_strings.push(format!("({}, {}, {})", index, year, month));

        match status.lock()
        {
            Ok(mut res) =>
            {
                if res.cancelled
                {
                    return Ok(timepoints)
                }
                
                res.current_progress = index;
            }

            Err(e) =>
            {
                return Err(Error::new(ErrorKind::Other, e.to_string()));
            }
        }
    }

    let batches = ((timepoint_strings.len() / BATCH_SIZE) + 1) as usize;

    match status.lock()
    {
        Ok(mut res) =>
        {
            if res.cancelled
            {
                return Ok(timepoints)
            }

            res.set_stage(4);
            res.current_progress = 0;
            res.total_work = batches as u32;
            res.discrete_progress = true;
        }

        Err(e) =>
        {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    }

    for batch in 0..batches
    {
        let batch_start = batch * BATCH_SIZE;
        let batch_end = std::cmp::min(batch_start + BATCH_SIZE, timepoint_strings.len());

        match connection.lock()
        {
            Ok(mut res) =>
            {
                if !res.insert_timepoints(&timepoint_strings[batch_start..batch_end])
                {
                    return Err(Error::new(ErrorKind::Other, "Failed to insert regions"));
                }
            }
    
            Err(e) =>
            {
                return Err(Error::new(ErrorKind::Other, e.to_string()));
            }
        }

        match status.lock()
        {
            Ok(mut res) =>
            {
                if res.cancelled
                {
                    return Ok(timepoints)
                }
                
                res.current_progress = batch as u32 + 1;
            }

            Err(e) =>
            {
                return Err(Error::new(ErrorKind::Other, e.to_string()));
            }
        }
    }

    Ok(timepoints)
}

fn import_weatherpoints(status: &Arc<Mutex<Status>>, connection: &Arc<Mutex<PGConnection>>, timepoints: Vec<Timepoint>, region_count: u32) -> Result<()>
{
    godot_print!("Importing Weatherpoints");

    match status.lock()
    {
        Ok(mut res) =>
        {
            if res.cancelled
            {
                return Ok(())
            }

            res.set_stage(5);
            res.current_progress = 0;
            res.total_work = timepoints.len() as u32;
            res.discrete_progress = true;
        }

        Err(e) =>
        {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    }

    let (sender, receiver) = sync_channel(4);
    let batchConnection = Arc::clone(connection);
    let handle = thread::spawn(move|| {
        batch_weatherpoints(receiver, batchConnection)
    });

    for (index, timepoint) in timepoints.into_iter().enumerate()
    {
        let mut points = Arc::new(Mutex::new(Vec::<String>::with_capacity(region_count as usize)));

        match timepoint
        {
            Timepoint(year, month) =>
            {
                let mut points = points.lock().unwrap();

                let suffix = format!("{}-{:02}", year, month);

                let mut file = BufReader::new(File::open(format!("large_data/weather_points/{}.bin", suffix))?);

                let mut header = [0u8; 1];
                file.read(&mut header)?;

                if header[0] != 0xCC
                {
                    return Err(Error::new(ErrorKind::InvalidData, format!("Weatherpoint File {}: invalid header", suffix)));
                }

                let mut timepoint_id_buffer = [0u8; 4];
                file.read(&mut timepoint_id_buffer)?;
                
                let timepoint_id = u32::from_le_bytes(timepoint_id_buffer);

                if (timepoint_id as usize) != index
                {
                    return Err(Error::new(ErrorKind::InvalidData, format!("Weatherpoint File {}: invalid ID", suffix)));
                }

                for regionID in 0..region_count
                {
                    match status.lock()
                    {
                        Ok(mut res) =>
                        {
                            if res.cancelled
                            {
                                return Ok(())
                            }
                        }
            
                        Err(e) =>
                        {
                            return Err(Error::new(ErrorKind::Other, e.to_string()));
                        }
                    }

                    let mut buffer = [0u8; 1];

                    file.read(&mut header)?;
                    let prec_mapped = u8::from_le_bytes(buffer);

                    file.read(&mut header)?;
                    let tmin = i8::from_le_bytes(buffer);

                    file.read(&mut header)?;
                    let tmax_mapped = i8::from_le_bytes(buffer);

                    let prec = (-255f32 * f32::ln(-((prec_mapped as f32) - 255f32) / 255f32)) as u8;
                    let tmax = (tmax_mapped + 20);

                    if tmin > tmax
                    {
                        godot_print!("Problem: {}, {}", tmin, tmax);
                    }

                    points.push(format!("({}, {}, {}, {}, {})", regionID, index, prec, tmin, tmax));
                }
            }
        }

        sender.send(points);

        match status.lock()
        {
            Ok(mut res) =>
            {
                if res.cancelled
                {
                    return Ok(())
                }

                res.current_progress = index as u32 + 1;
            }

            Err(e) =>
            {
                return Err(Error::new(ErrorKind::Other, e.to_string()));
            }
        }
    }

    match status.lock()
    {
        Ok(mut res) =>
        {
            if res.cancelled
            {
                return Ok(())
            }

            res.set_stage(6);
            res.discrete_progress = false;
        }

        Err(e) =>
        {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    }

    match handle.join()
    {
        Ok(res) =>
        {
            match res
            {
                Ok(_) => {}
                Err(err) =>
                {
                    return Err(Error::new(ErrorKind::Other, err.to_string()));
                }
            }
        }

        Err(_) =>
        {
            return Err(Error::new(ErrorKind::Other, "Weatherpoint batcher panicked!"));
        }
    }

    Ok(())
}


fn batch_weatherpoints(receiver: Receiver<Arc<Mutex<Vec<String>>>>, connection: Arc<Mutex<PGConnection>>) -> Result<()>
{
    loop
    {
        let data = match receiver.recv()
        {
            Ok(res) => res,
            Err(e) =>
            {
                return Err(Error::new(ErrorKind::Other, e.to_string()));
            }
        };
        let data = match data.lock()
        {
            Ok(res) => res,
            Err(e) =>
            {
                return Err(Error::new(ErrorKind::Other, e.to_string()));
            }
        };

        if data.len() == 0
        {
            break;
        }

        let batches = ((data.len() / BATCH_SIZE) + 1) as usize;

        for batch in 0..batches
        {
            let batch_start = batch * BATCH_SIZE;
            let batch_end = std::cmp::min(batch_start + BATCH_SIZE, data.len());
    
            match connection.lock()
            {
                Ok(mut res) =>
                {
                    if !res.insert_weatherpoints(&data[batch_start..batch_end])
                    {
                        return Err(Error::new(ErrorKind::Other, "Failed to insert regions"));
                    }
                }
        
                Err(e) =>
                {
                    return Err(Error::new(ErrorKind::Other, e.to_string()));
                }
            }
        }
    }

    Ok(())
}


fn import_countries(status: &Arc<Mutex<Status>>, connection: &Arc<Mutex<PGConnection>>) -> Result<()>
{
    godot_print!("Importing Countries");

    let countries: Vec<PathBuf> = fs::read_dir(Path::new("large_data/countries/"))?
        .filter_map(|country|
        {
            match country
            {
                Ok(entry) =>
                {
                    let is_file = entry.path().is_file();
                    let is_bin = entry.path().extension().unwrap_or_default() == "bin";

                    if is_file && is_bin
                    {
                        return Some(entry.path())
                    }

                    None
                }

                Err(_) => None
            }
        }).collect();

    match status.lock()
    {
        Ok(mut res) =>
        {
            if res.cancelled
            {
                return Ok(())
            }

            res.set_stage(7);
            res.current_progress = 0;
            res.total_work = countries.len() as u32;
            res.discrete_progress = true;
        }

        Err(e) =>
        {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    }

    for (index, country) in countries.into_iter().enumerate()
    {
        let mut file = BufReader::new(File::open(&country)?);

        let mut header = [0u8; 1];
        file.read(&mut header)?;

        if header[0] != 0xDD
        {
            return Err(Error::new(ErrorKind::InvalidData, format!("Country File {}: invalid header", &country.display())));
        }

        let mut iso_a3 = [0u8; 3];
        file.read(&mut iso_a3);
        let mut iso_a3 = std::str::from_utf8(&iso_a3).unwrap_or_default().to_string();
        iso_a3.make_ascii_uppercase();
        iso_a3.push_str(".bin");

        if !country.file_name().and_then(|f| f.to_str()).unwrap_or_default().ends_with(&iso_a3)
        {
            return Err(Error::new(ErrorKind::InvalidData, format!("Country File {}: invalid ID", &country.display())));
        }

        let mut name_length = [0u8; 4];
        file.read(&mut name_length)?;
        
        let name_length = u32::from_le_bytes(name_length);

        let mut name = Vec::<u8>::new();
        name.resize(name_length as usize, 0u8);
        file.read(&mut name);

        let name = std::str::from_utf8(&name).unwrap_or_default();

        let mut center_buf = [0u8; 2];

        file.read(&mut center_buf)?;
        let center_x = u16::from_le_bytes(center_buf);
        
        file.read(&mut center_buf)?;
        let center_y = u16::from_le_bytes(center_buf);

        let mut count = [0u8; 4];
        file.read(&mut count)?;
        
        let count = u32::from_le_bytes(count);

        let mut region_buf = [0u8; 4];
        let mut regions = Vec::<u32>::with_capacity(count as usize);

        for _ in 0..count
        {
            file.read(&mut region_buf)?;
            regions.push(u32::from_le_bytes(region_buf));
        }

        match connection.lock()
        {
            Ok(mut res) =>
            {
                if !res.insert_country(&iso_a3, &name, center_x as u32, center_y as u32, &regions)
                {
                    return Err(Error::new(ErrorKind::Other, "Failed to insert regions"));
                }
            }
    
            Err(e) =>
            {
                return Err(Error::new(ErrorKind::Other, e.to_string()));
            }
        }

        match status.lock()
        {
            Ok(mut res) =>
            {
                if res.cancelled
                {
                    return Ok(())
                }

                res.current_progress = index as u32 + 1;
            }

            Err(e) =>
            {
                return Err(Error::new(ErrorKind::Other, e.to_string()));
            }
        }
    }

    Ok(())
}

fn set_error(status: &Arc<Mutex<Status>>, error: Error)
{
    match status.lock()
    {
        Ok(mut res) =>
        {
            res.error = Some(error.to_string());
        }

        Err(_) => {}
    }
}
use erebos::instructions::*;
use erebos::error;
use std::collections::HashMap;
use std::fs::{DirEntry, File};
use std::io::{BufReader, BufWriter, Write};
use std::io::Read;
use std::io::Seek;
use std::path::Path;

macro_rules! fs_error
{
    ($($arg:tt)*) => 
    { 
        error!("FileSystem[TM]: {}", format!($($arg)*))
    }
}

#[repr(u8)]
#[derive(PartialEq)]
pub enum FSResult
{
    OK = 0,
    InvalidIndex,
    FileExists,
    FileDoesntExist,
    CouldntRead,
    CouldntWrite,
    RootDoesntExist,
    IsntDir,
    IsntFile,
    NotFound,
    Unknown
}
impl From<u8> for FSResult
{
    fn from(value: u8) -> Self
    {
        if(value < FSResult::Unknown.into())
        {
            unsafe { std::mem::transmute::<u8, FSResult>(value) }
        }
        else
        {
            FSResult::Unknown
        }
    }
}
impl From<FSResult> for u8
{
    fn from(value: FSResult) -> Self
    {
        unsafe { std::mem::transmute(value) }
    }
}

pub struct FS
{
    pub active: bool,
    pub file_loc: String,
    pub map: HashMap<u8, DirEntry>,
    pub file_count: u8,
}
impl Default for FS
{
    fn default() -> Self { Self::new() }
}
impl FS
{

    pub fn new() -> Self
    {
        Self
        {
            active: false,
            file_loc: String::new(),
            map: HashMap::new(),
            file_count: 0,
        }
    }

    pub fn IsActive(&self) -> bool { self.active }

    fn _check_active(&self) -> Result<(), Error> 
    {
        if(!self.IsActive()) { return Err(fs_error!("Is not active!")); }
        Ok(())
    }
    fn _check_index(&self, index: u8) -> FSResult
    {
        if(index == 0 || index > self.file_count)
        {
            FSResult::InvalidIndex
        }
        else
        {
            FSResult::OK
        }
    }



    pub fn Reindex(&mut self) -> Result<(), Error>
    {
        
        self._check_active()?;

        self.map.clear();

        if(!matches!(std::fs::exists(&self.file_loc), Ok(true)))
        {
            return Err(fs_error!("File location '{}' does not exist!", self.file_loc));
        }

        if(!Path::new(&self.file_loc).is_dir())
        {
            return Err(fs_error!("File location '{}' is not a directory!", self.file_loc));
        }

        let mut index: u8 = 0;
        for entry in match std::fs::read_dir(&self.file_loc)
            {
                Ok(d) => d,
                Err(e) => return Err(Error::fromio(e)),
            }
        {

            let entry = match entry
            {
                Ok(d) => d,
                Err(e) => return Err(Error::fromio(e)),
            };

            let meta = match entry.metadata()
            {
                Ok(d) => d,
                Err(e) => return Err(Error::fromio(e)),
            };

            if(!meta.is_file()) { continue; }

            index += 1;
            self.map.insert(index, entry);

        }

        self.file_count = index;

        Ok(())

    }
    pub fn GetFiles(&self) -> Result<u8, Error>
    {
        self._check_active()?;
        Ok(self.file_count)
    }

    pub fn CreateFile(&self, path: String) -> Result<FSResult, Error>
    {

        self._check_active()?;
        
        if( match std::fs::exists(&path)
            {
                Ok(b) => b,
                Err(e) => return Err(Error::fromio(e)),
            } ) 
        { return Ok(FSResult::FileExists); }

        if let Err(e) = std::fs::write(path, vec![])
        {
            return Err(Error::fromio(e));
        }

        Ok(FSResult::OK)

    }
    pub fn DeleteFile(&mut self, index: u8) -> Result<FSResult, Error>
    {
        
        self._check_active()?;

        let r = self._check_index(index);
        if(r != FSResult::OK) { return Ok(r); }

        let entry = match self.map.get(&index)
        {
            Some(e) => e,
            None => unreachable!(),
        };

        let path = entry.path();

        if(!path.exists())
        {
            return Err(fs_error!("DeleteFile(): index: {}, file: {} : File doesnt exist!", index, path.display()));
        }

        match std::fs::remove_file(path)
        {
            Ok(_) => {},
            Err(e) => return Err(Error::fromio(e)),
        }

        self.Reindex()?;

        Ok(FSResult::OK)
        
    }

    pub fn FileExists(&self, path: String) -> Result<u8, Error>
    {

        let b = match std::fs::exists(&path)
        {
            Ok(b) => b,
            Err(e) => return Err(Error::fromio(e)),
        };

        if(b) { Ok(1) }
        else  { Ok(0) }

    }

    pub fn GetSupDir(&self, path: String) -> Result<(FSResult, Option<String>), Error>
    {

        if let Err(e) = std::fs::exists(&path)
        {
            return Err(Error::fromio(e));
        }

        let p = Path::new(&path);
        
        Ok(match p.parent()
        {
            Some(p) => ( FSResult::OK, Some(p.to_string_lossy().to_string()) ),
            None => ( FSResult::NotFound, None ),
        })

    }

    pub fn QuickRead(&self, path: String) -> Result<(FSResult, Option<Vec<u8>>), Error>
    {

        let path = Path::new(&path);

        if(!path.exists())
        {
            return Ok(( FSResult::FileDoesntExist, None ));
        }

        if(!path.is_file())
        {
            return Ok(( FSResult::IsntFile, None ));
        }

        match std::fs::read(path)
        {
            Ok(v) => Ok(( FSResult::OK, Some(v) )),
            Err(e) => Err(Error::fromio(e))
        }

    }

    pub fn SetRoot(&mut self, path: String) -> Result<FSResult, Error>
    {

        if(!match std::fs::exists(&path)
        { 
            Ok(b) => b, 
            Err(e) => return Err(Error::fromio(e)),
        })
        {
            return Ok(FSResult::RootDoesntExist);
        }
        if(!Path::new(&path).is_dir())
        {
            return Ok(FSResult::IsntDir);
        }

        self.active = true;
        self.file_loc = path;

        Ok(FSResult::OK)

    }



    pub fn GetFileName(&self, index: u8) -> Result<(FSResult, Option<String>), Error>
    {

        self._check_active()?;
        
        let r = self._check_index(index);
        if(r != FSResult::OK) { return Ok((r, None)); }

        let entry = match self.map.get(&index)
        {
            Some(e) => e,
            None => unreachable!(),
        };

        let filename = match entry.file_name().into_string()
        {
            Ok(s) => s,
            Err(_) => return Err(fs_error!("Could not convert OSString '{:?}' into String!", entry.file_name())),
        };

        Ok(( FSResult::OK, Some(filename) ))

    }
    pub fn SetFileName(&mut self, index: u8, name: String) -> Result<FSResult, Error>
    {

        self._check_active()?;

        let r = self._check_index(index);
        if(r != FSResult::OK) { return Ok(r); }

        let entry = match self.map.get(&index)
        {
            Some(e) => e,
            None => unreachable!(),
        };

        let     oldpath = entry.path();
        let mut newpath = entry.path();
        newpath.set_file_name(name);

        if let Err(e) = std::fs::rename(oldpath, newpath)
        {
            return Err(Error::fromio(e));
        };

        self.Reindex()?;

        Ok(FSResult::OK)

    }

    pub fn GetFileLength(&self, index: u8) -> Result<(FSResult, Option<u32>), Error>
    {

        self._check_active()?;
        
        let r = self._check_index(index);
        if(r != FSResult::OK) { return Ok((r, None)); }

        let entry = match self.map.get(&index)
        {
            Some(e) => e,
            None => unreachable!(),
        };

        let meta = match entry.metadata()
        {
            Ok(m) => m,
            Err(e) => return Err(Error::fromio(e)),
        };

        let size = meta.len() as u32;

        Ok(( FSResult::OK, Some(size) ))

    }
    pub fn SetFileLength(&self, index: u8, length: u32) -> Result<FSResult, Error>
    {

        self._check_active()?;
        
        let r = self._check_index(index);
        if(r != FSResult::OK) { return Ok(r); }

        let entry = match self.map.get(&index)
        {
            Some(e) => e,
            None => unreachable!(),
        };

        let file = match File::open(entry.path())
        {
            Ok(f) => f,
            Err(e) => return Err(Error::fromio(e)),
        };

        if let Err(e) = file.set_len(length as u64)
        {
            return Err(Error::fromio(e));
        };

        Ok(FSResult::OK)

    }

    pub fn ReadFile(&self, index: u8) -> Result<(FSResult, Option<Vec<u8>>), Error>
    {

        self._check_active()?;
        
        let r = self._check_index(index);
        if(r != FSResult::OK) { return Ok((r, None)); }

        let entry = match self.map.get(&index)
        {
            Some(e) => e,
            None => unreachable!(),
        };

        let buffer = match std::fs::read(entry.path())
        {
            Ok(b) => b,
            Err(e) => return Err(Error::fromio(e)),
        };

        Ok(( FSResult::OK, Some(buffer) ))

    }
    pub fn ReadFileAt(&self, index: u8, pos: u32) -> Result<(FSResult, Option<u8>), Error>
    {

        self._check_active()?;
        
        let r = self._check_index(index);
        if(r != FSResult::OK) { return Ok((r, None)); }

        let entry = match self.map.get(&index)
        {
            Some(e) => e,
            None => unreachable!(),
        };

        let file = match File::open(entry.path())
        {
            Ok(f) => f,
            Err(e) => return Err(Error::fromio(e)),
        };

        let mut buffer: [u8; 1] = [0; 1];

        let mut reader = BufReader::new(file);
        if let Err(e) = reader.seek(std::io::SeekFrom::Start(pos as u64))
        {
            return Err(Error::fromio(e));
        }
        match reader.read(&mut buffer)
        {
            Ok(0) => return Ok(( FSResult::CouldntRead, None )),
            Err(e) => return Err(Error::fromio(e)),
            _ => {},
        };

        Ok(( FSResult::OK, Some(buffer[0]) ))

    }

    pub fn WriteFile(&self, index: u8, buffer: Vec<u8>) -> Result<FSResult, Error>
    {

        self._check_active()?;
        
        let r = self._check_index(index);
        if(r != FSResult::OK) { return Ok(r); }

        let entry = match self.map.get(&index)
        {
            Some(e) => e,
            None => unreachable!(),
        };

        let file = match File::open(entry.path())
        {
            Ok(f) => f,
            Err(e) => return Err(Error::fromio(e)),
        };

        let mut writer = BufWriter::new(file);
        if let Err(e) = writer.write(&buffer)
        {
            return Err(Error::fromio(e));
        };

        Ok(FSResult::OK)

    }
    pub fn WriteFileAt(&self, index: u8, pos: u32, val: u8) -> Result<FSResult, Error>
    {

        self._check_active()?;
        
        let r = self._check_index(index);
        if(r != FSResult::OK) { return Ok(r); }

        let entry = match self.map.get(&index)
        {
            Some(e) => e,
            None => unreachable!(),
        };

        let file = match File::open(entry.path())
        {
            Ok(f) => f,
            Err(e) => return Err(Error::fromio(e)),
        };

        let buffer: [u8; 1] = [val; 1];
        let mut writer = BufWriter::new(file);
        if let Err(e) = writer.seek(std::io::SeekFrom::Start(pos as u64))
        {
            return Err(Error::fromio(e));
        }
        match writer.write(&buffer)
        {
            Ok(0) => return Ok(FSResult::CouldntWrite),
            Err(e) => return Err(Error::fromio(e)),
            _ => {},
        };

        Ok(FSResult::OK)

    }

}

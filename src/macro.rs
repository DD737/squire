
#[macro_export]
macro_rules! error
{
    ($($arg:tt)*) => 
    { 
        squire::instructions::Error::from(format!($($arg)*))
        //std::io::Error::new(std::io::ErrorKind::Other, format!($($arg)*)) 
    }
}
#[macro_export]
macro_rules! error_in
{
    ($loc:tt,$($arg:tt)*) => 
    { 
        squire::instructions::Error::fromin(format!($($arg)*), $loc)
        //std::io::Error::new(std::io::ErrorKind::Other, format!($($arg)*)) 
    }
}

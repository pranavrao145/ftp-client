use std::error::Error;

use ftp_client::ftp_stream::FtpStream;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = FtpStream::connect("192.168.88.250:21").await?;
    stream.login("username", "password").await?;

    Ok(())
}

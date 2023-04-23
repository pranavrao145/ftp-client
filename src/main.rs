use dotenv::dotenv;
use std::{env, error::Error};

use ftp_client::ftp_stream::FtpStream;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let addr = env::var("FTP_CLIENT_SERVER_ADDRESS")?;
    let username = env::var("FTP_CLIENT_USERNAME")?;
    let password = env::var("FTP_CLIENT_PASSWORD")?;

    let mut stream = FtpStream::connect(addr.as_str()).await?;
    stream.login(username.as_str(), password.as_str()).await?;

    Ok(())
}

use std::error::Error;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::Mutex,
};

pub struct FtpStream {
    pub conn: Mutex<BufReader<TcpStream>>,
}

impl FtpStream {
    /// Attempt to connect to the server with the given address. Return an error
    /// if the connection is unsuccessful.
    ///
    /// * `addr`: the address to attempt to connect to
    pub async fn connect(addr: &str) -> Result<FtpStream, Box<dyn Error>> {
        let stream = TcpStream::connect(addr).await?;
        let connection = FtpStream {
            conn: Mutex::new(BufReader::new(stream)),
        };

        let mut line = String::new();
        let mut conn = connection.conn.lock().await;

        conn.read_line(&mut line).await?;

        let status_code: String = line.chars().take(3).collect();
        let status_code = status_code.parse::<i32>()?;

        // if we were not able to connect to the server
        if status_code != 220 {
            return Err("failed to connect to server".into());
        }

        drop(conn);

        Ok(connection)
    }

    /// Attempt to log in to the FTP server with the username and password provided. Return an
    /// error if the login is unsuccessful.
    ///
    /// * `username`: the username with which to make the login attempt
    /// * `password`: the password with which to make the login attempt
    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), Box<dyn Error>> {
        let mut conn = self.conn.lock().await;

        self.write_to_conn(format!("USER {}\r\n", username).as_str(), &mut conn)
            .await?;

        // get response status code to check if password needed
        let next_response_code = self.get_next_response_code(&mut conn).await?;

        // if a password is needed
        if next_response_code == 331 {
            self.write_to_conn(format!("PASS {}\r\n", password).as_str(), &mut conn)
                .await?;
        }

        // get response status code to check if password needed
        let next_response_code = self.get_next_response_code(&mut conn).await?;

        // if the login is not successful
        if next_response_code != 230 {
            return Err(format!("login failed with error code {}", next_response_code).into());
        }

        Ok(())
    }

    /// Writes the given message to the TCP connection
    ///
    /// * `message`: the message to write
    /// * `conn`: a mutable reference to the BufReader containing the TCPStream to write to
    pub async fn write_to_conn(
        &self,
        message: &str,
        conn: &mut BufReader<TcpStream>,
    ) -> Result<(), Box<dyn Error>> {
        let stream = conn.get_mut();
        stream.write_all(message.as_bytes()).await?;
        stream.flush().await?;

        Ok(())
    }

    /// Returns the status code of the next message to be read in the stream
    ///
    /// * `conn`: a mutable reference to the BufReader containing the TCPStream to read from
    pub async fn get_next_response_code(
        &self,
        conn: &mut BufReader<TcpStream>,
    ) -> Result<i32, Box<dyn Error>> {
        let mut line = String::new();
        conn.read_line(&mut line).await?;

        let status_code: String = line.chars().take(3).collect();
        let status_code: i32 = status_code.parse()?;

        Ok(status_code)
    }

    /// Returns an int with the status code of the next message as well as the content of the next message.
    ///
    /// * `conn`: a mutable reference to the BufReader containing the TCPStream to read from
    pub async fn get_next_message(&self, conn: &mut BufReader<TcpStream>) -> Result<(i32, String), Box<dyn Error>> {
        let mut first_line = String::new();
        let mut rest = String::new();
        
        // read first line
        conn.read_line(&mut first_line).await?;

        let status_code: String = first_line.chars().take(3).collect();

        loop {
            let mut next_line = String::new();
            conn.read_line(&mut next_line).await?;

            rest.push_str("\n");
            rest.push_str(&next_line);

            let status_code_with_space = format!("{} ", status_code);

            if next_line.starts_with(&status_code_with_space) {
                break;
            }
        }

        let final_msg = format!("{}{}", first_line, rest);
        let status_code: i32 = status_code.parse()?;

        Ok((status_code, final_msg))
    }
}

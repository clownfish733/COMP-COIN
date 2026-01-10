use std::net::IpAddr;

use anyhow::{anyhow, Result};
use get_if_addrs::get_if_addrs;

pub fn get_local_ip() -> Result<IpAddr>{
    let interfaces = get_if_addrs()?;

    for iface in interfaces{
        if !iface.is_loopback() && iface.ip().is_ipv4() {
            return Ok(iface.ip())
        }
    }
    Err(anyhow!("No local ip address found"))

}

pub async fn get_global_ip() -> Result<IpAddr>{
    let response = reqwest::get("https://api.ipify.org").await?;
    let ip_string = response.text().await?;
    let ip: IpAddr = ip_string.trim().parse()?;
    Ok(ip)
}


#[cfg(test)]
mod tests{
    use super::*;

    #[tokio::test]
    async fn ip_test() {
        println!("{:#?}", get_local_ip());
        println!("{:#?}", get_global_ip().await);
    }
}
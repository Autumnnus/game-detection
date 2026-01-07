use wmi::{COMLibrary, WMIConnection};
use serde::Deserialize;
use crate::models::ProcessInfo;

#[allow(non_camel_case_types, non_snake_case)]
#[derive(Deserialize, Debug)]
struct Win32_Process {
    Name: String,
    ProcessId: u32,
}

pub struct SystemScanner {
    wmi_con: WMIConnection,
}

impl SystemScanner {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let com_con = COMLibrary::new()?;
        let wmi_con = WMIConnection::new(com_con.into())?;
        Ok(Self { wmi_con })
    }

    pub fn get_running_processes(&self) -> Result<Vec<ProcessInfo>, Box<dyn std::error::Error>> {
        let processes: Vec<Win32_Process> = self.wmi_con.query()?;
        
        Ok(processes.into_iter().map(|p| ProcessInfo {
            name: p.Name,
            pid: p.ProcessId,
        }).collect())
    }
}

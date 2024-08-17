use std::str::FromStr;

use anyhow::{Context, Error, Result};

pub const WINDOW_FORMAT: &str = "#{#W,#{?window_active,1,},#{window_activity},#{window_panes}}";

#[derive(Debug)]
pub struct Window {
    name: String,
    is_active: bool,
    last_active: u64,
    //panes: Vec<String>, TBD
    panes_number: usize,
}

impl FromStr for Window {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<_> = s.split(',').collect();

        assert_eq!(
            parts.len(),
            4,
            "should be 4 parts in list-windows format str"
        );

        Ok(Window {
            name: parts[0].into(),
            is_active: parts[1] == "1",
            last_active: parts[2]
                .parse()
                .context("error parsing window last_active")?,
            //panes:
            panes_number: parts[3]
                .parse()
                .context("error parsing window panes_number")?,
        })
    }
}
